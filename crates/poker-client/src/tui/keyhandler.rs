use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::{AppState, InputType, UIScreenMode};

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

    // This will just be a long match...
    // Try to keep logic out as much as possible.
    // Move to functions if code blocks for a single key
    // get longer than few lines
    match state.ui_state.screen_mode {
        UIScreenMode::Main => match key.code {
            KeyCode::Char('s') => state.ui_state.screen_mode = UIScreenMode::Session,
            KeyCode::Char('U') => set_input(state, InputType::Server),
            KeyCode::Char('N') => set_input(state, InputType::UserName),
            _ => {}
        },
        UIScreenMode::Session => {}
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
