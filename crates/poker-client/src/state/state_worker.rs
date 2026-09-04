// This mod has the logic for the worker
// from the apps point of view

// the functions for the worker itself
// are defined int he worker.rs file!

use std::time::{Duration, Instant};

use crate::worker::{WorkerEvent, WorkerRequest};

use super::AppState;

const HEALTH_CHECK_MILLI: Duration = Duration::from_millis(8000);

/// Main function that deals with the worker
/// Checks the queue and handles results
pub fn check_worker_logic(state: &mut AppState) {
    // Check periodic events
    // only check if server is NOT active
    if !state.worker.worker_active {
        // server connection check
        // We only check if we actually have a url
        if let Some(server_url) = state.server_url.as_ref()
            && state.time_last_server_check.elapsed() >= HEALTH_CHECK_MILLI
        {
            // Create a worker check server connection event
            state
                .worker
                .tasks
                .push_back(WorkerRequest::CheckServerConnection(server_url.clone()));
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

fn handle_worker_event(state: &mut AppState, event: WorkerEvent) {
    match event {
        WorkerEvent::ServerHealth { ok } => {
            // Reset server check time
            state.time_last_server_check = Instant::now();
            state.server_connected = ok;
        }
        WorkerEvent::Error(error_msg) => {
            state.worker_error = Some(error_msg.clone());
        }
    }
}
