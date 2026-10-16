// This mod has the logic for the worker
// from the apps point of view

// the functions for the worker itself
// are defined int he worker.rs file!

use std::time::{Duration, Instant};

use crate::worker::{WorkerEvent, WorkerRequest};

use super::{AppState, reset_session_info};

const HEALTH_CHECK_MILLI: Duration = Duration::from_millis(8000);
// item list
const ITEM_FETCH_MILLI: Duration = Duration::from_millis(5000);
// selected item info
// Bit of a weird time to prevent overlaps with other periods
const ITEM_INFO_FETCH_MILLI: Duration = Duration::from_millis(2800);

/// Main function that deals with the worker
/// Checks the queue and handles results
pub fn check_worker_logic(state: &mut AppState) {
    // Check periodic events
    // only check if server is NOT active
    if !state.worker.worker_active {
        // server connection check
        // We always check! Also if we do not have a url
        // so this event ALSO functions as a heartbeat for the worker
        if state.time_last_server_check.elapsed() >= HEALTH_CHECK_MILLI {
            // Create a worker check server connection event
            state
                .worker
                .tasks
                .push_back(WorkerRequest::CheckServerConnection(
                    state.server_url.clone(),
                ));
        }

        if state.time_last_item_fetch.elapsed() >= ITEM_FETCH_MILLI {
            // We only check if we have an active session
            if !state.selected_session.is_empty() {
                // It should be impossible to NOT have
                // a server url here...
                // But check anyway
                if let Some(url) = &state.server_url {
                    state.worker.tasks.push_back(WorkerRequest::GetItems(
                        url.clone(),
                        state.selected_session.clone(),
                    ));
                }
            } else {
                // reset timer, do not check again for a while
                state.time_last_item_fetch = Instant::now();
            }
        }

        if state.time_last_item_info_fetch.elapsed() >= ITEM_INFO_FETCH_MILLI {
            if !state.selected_session.is_empty() && state.selected_item != 0 {
                if let Some(url) = &state.server_url {
                    state.worker.tasks.push_back(WorkerRequest::GetItemInfo(
                        url.clone(),
                        state.selected_session.clone(),
                        state.selected_item,
                    ));
                }
            } else {
                // reset timer, otherwise we keep checking
                state.time_last_item_info_fetch = Instant::now();
            }
        }
    }

    // Check if results
    if state.worker.worker_active
        && let Ok(event) = state.worker.receiver.try_recv()
    {
        handle_worker_event(state, event);
        state.worker.worker_active = false;
    }
    // If error from receiver, this probably means is empty

    // Check new events
    if !state.worker.worker_active {
        if let Some(request) = state.worker.tasks.pop_front() {
            match state.worker.sender.send(request) {
                Ok(()) => state.worker.worker_active = true,
                Err(e) => {
                    state.worker_error =
                        Some(format!("Error sending request to worker. Message: {e}"))
                }
            }
        }
    }
}

/// This handles responses from the worker
/// Adjust the state as needed, based on the response
/// from the worker
fn handle_worker_event(state: &mut AppState, event: WorkerEvent) {
    match event {
        WorkerEvent::ServerHealth { ok } => {
            // Reset server check time
            state.time_last_server_check = Instant::now();
            state.server_connected = ok;
        }
        WorkerEvent::Error(error_msg) => {
            state.worker_error = Some(error_msg.clone());
            // reset timers, or we can get looped errors
            state.time_last_item_fetch = Instant::now();
            state.time_last_item_info_fetch = Instant::now();
        }
        WorkerEvent::Sessions { sessions } => {
            state.available_sessions = sessions;
        }
        WorkerEvent::CreatedSession(session) => {
            reset_session_info(state);
            state.selected_session = session;
        }
        WorkerEvent::DeletedSession => {
            // On successful delete, simply clear the selected session
            state.selected_session.clear();
        }

        // New item-related events: no-ops for now (UI/state not wired yet)
        WorkerEvent::Items { items } => {
            // We divide the received items in active and closed items
            // and store in state
            state.active_items.clear();
            state.closed_items.clear();
            for item in items.iter() {
                if item.done {
                    state.closed_items.push(item.clone());
                } else {
                    state.active_items.push(item.clone());
                }
            }

            state.time_last_item_fetch = Instant::now();
        }
        WorkerEvent::ItemInfo { item } => {
            // We need to update our users list when we see votes
            for username in item.votes.keys() {
                if username != &state.user_name && !state.known_users.contains(username) {
                    state.known_users.push(username.clone());
                }
                // To prevent superfluous sort calls,
                // we only sort when we have found a new user
                // This is more efficient in the long run :)
                state.known_users.sort_by_key(|a| a.to_lowercase());
            }
            state.selected_item_info = Some(item);
            state.time_last_item_info_fetch = Instant::now();
        }
        WorkerEvent::CreatedItem {} => {
            // Get required info and update list
            if let Some(url) = &state.server_url {
                state.worker.tasks.push_back(WorkerRequest::GetItems(
                    url.clone(),
                    state.selected_session.clone(),
                ));
            }
        }
        WorkerEvent::ItemRevealed => {}
        WorkerEvent::ItemFinished => {
            // We might want to quickly update item list
            // So send a request
            if let Some(url) = &state.server_url {
                state.worker.tasks.push_back(WorkerRequest::GetItems(
                    url.clone(),
                    state.selected_session.clone(),
                ));
            }
        }
        WorkerEvent::ItemVoted => {}
        WorkerEvent::AzureItem(awi) => {
            // Update the hashmap
            state.azure_item_cache.insert(awi.id, awi);
        }
    }
}
