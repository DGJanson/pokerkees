// Since the app is simple, we put all our state in a single struct

use core::fmt;
use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use poker_shared::ItemInfo;
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

    // session
    pub selected_session: String,
    pub available_sessions: Vec<String>,

    // item
    pub selected_item: usize,
    pub active_items: Vec<ItemInfo>,
    pub closed_items: Vec<ItemInfo>,

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
}

/// The possible values of input type
/// Used to set values in the program
pub enum InputType {
    None,
    Server,
    UserName,
    NewSession,
    NewItemId,
    NewItemText,
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "<None>"),
            Self::Server => write!(f, "Server Url"),
            Self::UserName => write!(f, "User name"),
            Self::NewSession => write!(f, "Session name"),
            Self::NewItemId => write!(f, "Item ID"),
            Self::NewItemText => write!(f, "Item (short) description"),
        }
    }
}

pub struct WorkerState {
    pub tasks: VecDeque<WorkerRequest>,
    sender: Sender<WorkerRequest>,
    receiver: Receiver<WorkerEvent>,
    pub worker_active: bool,
}

pub fn init_state(
    name: String,
    url: Option<Url>,
    sender: Sender<WorkerRequest>,
    receiver: Receiver<WorkerEvent>,
) -> AppState {
    let ui_state = UIState {
        screen_mode: UIScreenMode::Main,
        input_type: InputType::None,
        input_value: String::new(),
        selected_index: 0,
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
        selected_session: String::new(),
        available_sessions: Vec::new(),
        selected_item: 0,
        active_items: Vec::new(),
        closed_items: Vec::new(),
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

/// Set active session based on the
/// selected available session
pub fn select_session(state: &mut AppState) {
    // We assume this function is only called from
    // a sensible location in the program.
    // Hence we only check program failure states
    // not logical failure states
    if state.ui_state.selected_index < state.available_sessions.len() {
        state.selected_session = state
            .available_sessions
            .get(state.ui_state.selected_index)
            .cloned()
            .unwrap_or_default();
    }
    // We also send a request to fetch items
    if let Some(url) = &state.server_url {
        state.worker.tasks.push_back(WorkerRequest::GetItems(
            url.clone(),
            state.selected_session.clone(),
        ));
    }
}
