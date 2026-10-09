use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
// This mod takes care of all input in the app
use url::Url;

use crate::{
    state::{AppState, InputType, UIScreenMode},
    worker::WorkerRequest,
};

/// Generic function for activating input
/// Sets basic settings for input, like UIState
/// And possibly sets up some specifics, depending on input type
pub fn set_input(state: &mut AppState, input_type: InputType) {
    state.ui_state.screen_mode = UIScreenMode::Input;
    match input_type {
        // None is unlikely here
        InputType::None => {}
        InputType::Server => {
            // Init with server url, but it could be None!
            state.ui_state.input_value = match state.server_url.as_ref() {
                None => String::new(),
                Some(url) => url.to_string(),
            }
        }
        InputType::UserName => {
            state.ui_state.input_value = state.user_name.clone();
        }
        InputType::NewSession => {
            state.ui_state.input_value = String::new();
        }
        InputType::NewItemId => {
            state.ui_state.input_value = String::new();
        }
        InputType::NewItemText => {
            state.ui_state.input_value = String::new();
        }
    }
    state.ui_state.input_type = input_type;
}

/// Call this function after user confirms input
/// It sets the correct value
pub fn handle_confirm_input(state: &mut AppState) {
    match state.ui_state.input_type {
        InputType::None => {}
        InputType::Server => {
            // In case of server, we need to try parse it to url
            let new_server_url = match Url::parse(state.ui_state.input_value.as_str()) {
                Ok(u) => Some(u),
                Err(e) => {
                    state.app_error = Some(format!("Could not parse url, because: {e:?}"));
                    None
                }
            };
            state.server_url = new_server_url;
            // Send a check connection event right away
            state
                .worker
                .tasks
                .push_back(WorkerRequest::CheckServerConnection(
                    state.server_url.clone(),
                ));
            // Reset screen state
            reset_input(state);
        }
        InputType::UserName => {
            state.user_name = state.ui_state.input_value.clone();
            // Reset screen state
            reset_input(state);
        }
        InputType::NewSession => {
            // In this case we only send the create request to server
            // After this succeeds, we activate the session in the app
            if let Some(url) = &state.server_url {
                state.worker.tasks.push_back(WorkerRequest::CreateSession(
                    url.clone(),
                    state.ui_state.input_value.clone(),
                ));
            }
            // Reset screen state
            reset_input(state);
        }
        InputType::NewItemId => {
            // Continue to input for item desc
            // Store item Id in selecten item for the meantime
            // We are quite sure it is a numeric value, since we
            // only accept digit chars
            let item_nr: usize = state.ui_state.input_value.parse().unwrap_or_default();
            if item_nr == 0 {
                state.app_error = Some(String::from("Item nr can not be zero"));
                reset_input(state);
                return;
            }
            state.selected_item = item_nr;
            set_input(state, InputType::NewItemText);
        }
        InputType::NewItemText => {
            // Send a post request to the server to create the item
            // Then return to main screen
            // It should fill with info on the item
            let item_desc = if state.ui_state.input_value.is_empty() {
                None
            } else {
                Some(state.ui_state.input_value.clone())
            };
            if let Some(url) = &state.server_url {
                state.worker.tasks.push_back(WorkerRequest::CreateItem(
                    url.clone(),
                    state.selected_session.clone(),
                    state.selected_item,
                    item_desc,
                ));
            }
            // Reset screen state
            reset_input(state);
        }
    }
}

pub fn reset_input(state: &mut AppState) {
    state.ui_state.screen_mode = UIScreenMode::Main;
    state.ui_state.input_value = String::new();
    state.ui_state.selected_index = 0;
}

/// This function decides what to do with the user input (key)
/// The confirm or escape keys are already handled in the main key handler
/// It could depend on the type of key input what we accept
/// Some general rules (for specific see the following functions)
/// - We only concern ourselves with the last char of the input value
///   either pop it (remove) or append to it. No mid input changes
/// - No linebreaks (ie no textarea like input)
/// - The general accept is ascii_graphic
///   but some input types (urls) may be more specific
pub fn input_handle_key(key: KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Backspace => {
            state.ui_state.input_value.pop();
        }
        KeyCode::Char(c) => {
            // Decide if we would like to add char to input value
            if accept_char(c, key.modifiers, &state.ui_state.input_type) {
                state.ui_state.input_value.push(c);
            }
        }
        // ignore tabs, arrows, function keys, etc.
        _ => {}
    }
}

// Decide whether a character should be appended.
// Keep this readable and conservative; expand later if needed.
fn accept_char(c: char, modifiers: KeyModifiers, input_type: &InputType) -> bool {
    // Ignore control/alt combos (e.g., Ctrl+C, Alt+something)
    if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::ALT) {
        return false;
    }

    match input_type {
        // URLs: allow RFC3986-ish characters (unreserved + reserved + %)
        // and disallow space to save the user from parse errors.
        InputType::Server => is_allowed_url_char(c),
        // Username: letters, digits, space and a few commonly-used separators
        InputType::UserName => is_allowed_username_char(c),
        // Numeric input
        InputType::NewItemId => is_allowed_numeric(c),
        // General fallback – allow printable ASCII and space
        _ => is_general_allowed_char(c),
    }
}

fn is_general_allowed_char(c: char) -> bool {
    c == ' ' || c.is_ascii_graphic()
}

fn is_allowed_numeric(c: char) -> bool {
    c.is_ascii_digit()
}

// Per RFC 3986: unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
// reserved = ":" / "/" / "?" / "#" / "[" / "]" / "@" / "!" / "$" / "&"
//            "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
// Also allow '%' for percent-encoding.
fn is_allowed_url_char(c: char) -> bool {
    if c.is_ascii_alphanumeric() {
        return true;
    }
    matches!(
        c,
        ':' | '/'
            | '?'
            | '#'
            | '['
            | ']'
            | '@'
            | '!'
            | '$'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | ';'
            | '='
            | '%'
            | '.'
            | '-'
            | '_'
            | '~'
    )
}

fn is_allowed_username_char(c: char) -> bool {
    // Allow unicode letters/digits (c.is_alphanumeric), spaces, and a few separators.
    // If you want to restrict to ASCII only, replace is_alphanumeric with is_ascii_alphanumeric.
    c.is_alphanumeric() || c == ' ' || matches!(c, '.' | '-' | '_' | '@')
}
