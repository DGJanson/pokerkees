use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    state::{AppState, InputType, UIScreenMode, reset_sessions, select_session},
    worker::WorkerRequest,
};

use super::inputhandler::{handle_confirm_input, input_handle_key, reset_input, set_input};

pub fn handle_key(key: KeyEvent, state: &mut AppState) {
    // We have seperate key handling based ONLY
    // on ui states.
    // That should be the only decider

    // ctrl + q should always exit the app
    if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL {
        state.should_exit = true;
        return;
    }

    // If there is an error message, we simply accept all key input
    // to remove it
    if state.app_error.is_some() {
        // remove and return
        state.app_error = None;
        return;
    }
    if state.worker_error.is_some() {
        // remove and return
        state.worker_error = None;
        return;
    }

    // This will just be a long match...
    // Try to keep logic out as much as possible.
    // Move to functions if code blocks for a single key
    // get longer than few lines
    match state.ui_state.screen_mode {
        UIScreenMode::Main => match key.code {
            KeyCode::Char('s') => {
                state.ui_state.screen_mode = UIScreenMode::Session;
                reset_sessions(state);
                // If we have a server url we can get sessions
                if let Some(url) = &state.server_url {
                    state
                        .worker
                        .tasks
                        .push_back(WorkerRequest::GetAvailableSessions(url.clone()));
                }
            }
            KeyCode::Char('U') => set_input(state, InputType::Server),
            KeyCode::Char('N') => set_input(state, InputType::UserName),
            // Now we check state and send key to appropriate handler
            KeyCode::Char('C') => {
                // Only makes sense when we have a session selected
                if !state.selected_session.is_empty() {
                    set_input(state, InputType::NewItemId);
                }
            }

            // select item
            KeyCode::Char('j') => {
                let length = state.active_items.len();
                if state.ui_state.selected_index + 1 < length {
                    state.ui_state.selected_index += 1;
                }
            }
            KeyCode::Char('k') => {
                if state.ui_state.selected_index > 0 {
                    state.ui_state.selected_index -= 1;
                }
            }
            _ => {}
        },
        UIScreenMode::Session => match key.code {
            KeyCode::Esc => {
                state.ui_state.screen_mode = UIScreenMode::Main;
                state.ui_state.selected_index = 0;
            }
            KeyCode::Char('j') => {
                if state.ui_state.selected_index + 1 < state.available_sessions.len() {
                    state.ui_state.selected_index += 1;
                }
            }
            KeyCode::Char('k') => {
                if state.ui_state.selected_index > 0 {
                    state.ui_state.selected_index -= 1;
                }
            }
            KeyCode::Enter => {
                select_session(state);
                state.ui_state.screen_mode = UIScreenMode::Main;
            }
            KeyCode::Char('C') => {
                set_input(state, InputType::NewSession);
            }
            KeyCode::Char('D') => {
                if let Some(url) = &state.server_url {
                    state.worker.tasks.push_back(WorkerRequest::DeleteSession(
                        url.clone(),
                        state.selected_session.clone(),
                    ));
                }
                state.ui_state.screen_mode = UIScreenMode::Main;
            }
            _ => {}
        },
        UIScreenMode::Input => match key.code {
            KeyCode::Esc => {
                // This will also reset the screen_mode
                reset_input(state);
            }
            KeyCode::Enter => {
                // This will also reset the screen mode
                handle_confirm_input(state);
            }

            // Offload the rest of the handling to the input script
            _ => input_handle_key(key, state),
        },
    }
}
