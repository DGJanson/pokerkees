// Since the app is simple, we put all our state in a single struct

use core::fmt;
use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use poker_shared::{ItemInfo, PokerItem};
use url::Url;

use crate::worker::{WorkerEvent, WorkerRequest};

pub mod state_worker;

pub struct AppState {
    // Basic stuff
    pub user_name: String,
    pub server_url: Option<Url>,

    // server
    pub server_connected: bool,
    pub time_last_server_check: Instant,
    pub time_last_item_fetch: Instant,
    pub time_last_item_info_fetch: Instant,

    // session
    pub selected_session: String,
    pub available_sessions: Vec<String>,
    /// known users, except self!
    pub known_users: Vec<String>,

    // item
    pub selected_item: usize,
    pub selected_item_info: Option<PokerItem>,
    pub active_items: Vec<ItemInfo>,
    pub closed_items: Vec<ItemInfo>,
    // item to temp store id whilst creating
    pub create_item_id: usize,

    // Azure integration
    pub azure_org: Option<String>,
    pub azure_project: Option<String>,
    pub azure_item_cache: HashMap<usize, AzureWorkItem>,

    // App flow logic
    pub should_exit: bool,

    // ui state
    pub ui_state: UIState,

    // worker state
    // ie. the stuff we need to manage the worker
    pub worker: WorkerState,

    // errors
    pub worker_error: Option<String>,
    pub app_error: Option<String>,
}

pub struct UIState {
    // The "state" of the screen
    // corresponds broadly to selected mode
    // by user. Eg. select session, read item, etc.
    pub screen_mode: UIScreenMode,
    pub input_type: InputType,
    pub input_value: String,
    // Used for selecting from lists
    // eg. sessions, items, etc
    // Note that we reuse this for multiple screens!
    pub selected_index: usize,
    // Scroller,
    // We only scroll vertically
    pub scroll: u16,
}

#[derive(PartialEq)]
pub enum UIScreenMode {
    // The main state of the UI
    // Draw general info
    Main,
    // The user has selected session mode
    // for manipulating a session
    Session,
    // The user has selected input mode
    // Used in combination with tje input_type
    // from the UIState to manage the input
    Input,
    // The screen in which a user can interact
    // with an item. Such as casting a vote
    // or revealing.
    Item,
}

/// The possible values of input type
/// Used to set values in the program
pub enum InputType {
    None,
    Server,
    UserName,
    AzureOrg,
    AzureProject,
    NewSession,
    NewItemId,
    NewItemText,
    Vote,
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "<None>"),
            Self::Server => write!(f, "Server Url"),
            Self::UserName => write!(f, "User name"),
            Self::AzureOrg => write!(f, "Azure Organisation"),
            Self::AzureProject => write!(f, "Azure Project"),
            Self::NewSession => write!(f, "Session name"),
            Self::NewItemId => write!(f, "Item ID"),
            Self::NewItemText => write!(f, "Item (short) description"),
            Self::Vote => write!(f, "Vote"),
        }
    }
}

pub struct WorkerState {
    pub tasks: VecDeque<WorkerRequest>,
    sender: Sender<WorkerRequest>,
    receiver: Receiver<WorkerEvent>,
    pub worker_active: bool,
}

#[derive(Debug, Clone)]
pub struct AzureWorkItem {
    pub id: usize,
    pub title: String,
    pub state: String,
    pub assigned_to: Option<String>,
    pub tags: Vec<String>,
    pub description: String,
    pub acceptance_criteria: String,
    pub created_date: String,
    pub last_fetched_at: Instant,
}

pub fn init_state(
    name: String,
    url: Option<Url>,
    sender: Sender<WorkerRequest>,
    receiver: Receiver<WorkerEvent>,
    azure_org: Option<String>,
    azure_project: Option<String>,
) -> AppState {
    let ui_state = UIState {
        screen_mode: UIScreenMode::Main,
        input_type: InputType::None,
        input_value: String::new(),
        selected_index: 0,
        scroll: 0,
    };

    let worker_state = WorkerState {
        tasks: VecDeque::new(),
        sender,
        receiver,
        worker_active: false,
    };

    AppState {
        user_name: name,
        server_url: url,
        server_connected: false,
        time_last_server_check: Instant::now(),
        time_last_item_fetch: Instant::now(),
        time_last_item_info_fetch: Instant::now(),
        selected_session: String::new(),
        available_sessions: Vec::new(),
        known_users: Vec::new(),
        selected_item: 0,
        selected_item_info: None,
        active_items: Vec::new(),
        closed_items: Vec::new(),
        create_item_id: 0,
        azure_org,
        azure_project,
        azure_item_cache: HashMap::new(),
        should_exit: false,
        ui_state,
        worker: worker_state,
        worker_error: None,
        app_error: None,
    }
}

// !! -- State change functions -- !!
pub fn reset_sessions(state: &mut AppState) {
    state.available_sessions.clear();
    state.ui_state.selected_index = 0;
}

/// Resets all information related to a session
/// use this to prevent info, like users, items,
/// leaking between sessions, when a user changes
/// sessions in the client.
pub fn reset_session_info(state: &mut AppState) {
    state.active_items.clear();
    state.closed_items.clear();
    state.known_users.clear();
    state.selected_item = 0;
    state.selected_item_info = None;
    state.ui_state.selected_index = 0;
}

/// Set active session based on the
/// selected available session
pub fn select_session(state: &mut AppState) {
    // We assume this function is only called from
    // a sensible location in the program.
    // Hence we only check program failure states
    // not logical failure states
    if state.ui_state.selected_index < state.available_sessions.len() {
        reset_session_info(state);
        state.selected_session = state
            .available_sessions
            .get(state.ui_state.selected_index)
            .cloned()
            .unwrap_or_default();
        // We also send a request to fetch items
        if let Some(url) = &state.server_url {
            state.worker.tasks.push_back(WorkerRequest::GetItems(
                url.clone(),
                state.selected_session.clone(),
            ));
        }
    }
}

/// Sets the selected item as active
pub fn select_item(state: &mut AppState) {
    // try to figure out which item is selected
    // There can be a condition in which the item list is
    // refreshed, changing indeces between selecting and enter
    // key press.
    // This seems highly unlikely, and we ignore for now
    if state.ui_state.selected_index < state.active_items.len() {
        if let Some(item_info) = state.active_items.get(state.ui_state.selected_index) {
            // update ui stuff
            state.selected_item = item_info.id;
            state.ui_state.screen_mode = UIScreenMode::Item;
            state.ui_state.scroll = 0;
            // We can fetch extra info on item now
            if let Some(url) = &state.server_url {
                state.worker.tasks.push_back(WorkerRequest::GetItemInfo(
                    url.clone(),
                    state.selected_session.clone(),
                    item_info.id,
                ));
            }
            // Also check if we want to get info from azure
            // First check if azure config complete
            if state.azure_org.is_some() && state.azure_project.is_some() {
                // There are two conditions in which we fetch:
                // - no info in cache yet
                // - stale info in cache
                let mut get_from_azure = false;
                if !state.azure_item_cache.contains_key(&item_info.id) {
                    get_from_azure = true;
                } else if let Some(awi) = state.azure_item_cache.get(&item_info.id) {
                    if awi.last_fetched_at.elapsed() >= Duration::from_secs(60) {
                        get_from_azure = true;
                    }
                }

                if get_from_azure {
                    // we are sure the org and project are filled
                    let azure_o = state.azure_org.as_deref().unwrap_or_default().to_string();
                    state
                        .worker
                        .tasks
                        .push_back(WorkerRequest::AzureItem(azure_o, item_info.id));
                }
            }
        }
    }
}
