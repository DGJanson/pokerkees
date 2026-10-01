// Code for the IO worker
// This is a single thread on which will perform IO
// in the background.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use handlers::sessions::parse_get_sessions;
use reqwest::blocking::Client;
use url::Url;

mod handlers;

// TODO, clean this
// this is raw LLM output

// UI -> Worker
// Note it is best to clone parameters to events,
// since they will be passed to other thread
#[derive(Debug, Clone)]
pub enum WorkerRequest {
    /// Check connection with server, using String parm as url
    CheckServerConnection(Option<Url>),
    GetAvailableSessions(Url),
    CreateSession(Url, String),
    DeleteSession(Url, String),
}

// Worker -> UI
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    ServerHealth { ok: bool },
    Sessions { sessions: Vec<String> },
    // Create the name of created session
    // so we can update state in frontend
    CreatedSession(String),
    // Deleted a session successfully
    DeletedSession,
    // In case of any error, simply return
    Error(String),
}

pub fn spawn_worker() -> (
    mpsc::Sender<WorkerRequest>,
    mpsc::Receiver<WorkerEvent>,
    thread::JoinHandle<()>,
) {
    let (tx_req, rx_req) = mpsc::channel::<WorkerRequest>();
    let (tx_evt, rx_evt) = mpsc::channel::<WorkerEvent>();

    let handle = thread::spawn(move || {
        // Create a blocking request client we can use in this worker
        // We only need one, since we are sync and single
        let client = Client::new();

        // Wait for requests, with a timeout of 30 secs, so that the thread
        // can disposed off in the event the frontend fails
        // we expect health calls every (max) 10 secs or so,
        // so the timeout should never be reached in case of
        // normal app use
        loop {
            match rx_req.recv_timeout(Duration::from_secs(30)) {
                // Handle request
                Ok(req) => {
                    let response = handle_request(req, &client);
                    // send the response back to app
                    // If err, the channel is closed
                    // and we can also close the worker
                    if tx_evt.send(response).is_err() {
                        break;
                    }
                }
                // Exit states
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    log::warn!("Worker timed out. Closing");
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    log::info!("All senders disconnected from worker. Closing");
                    break;
                }
            }
        }
    });
    log::info!("Spawned worker thread");

    (tx_req, rx_evt, handle)
}

fn handle_request(request: WorkerRequest, client: &Client) -> WorkerEvent {
    match request {
        // Check if we can connect to server
        WorkerRequest::CheckServerConnection(url) => {
            // simple check, do inline
            check_server_connection(url, client)
        }
        WorkerRequest::GetAvailableSessions(url) => {
            // simply get a list from server
            get_sessions(url, client)
        }
        WorkerRequest::CreateSession(url, session_name) => {
            create_session(url, session_name.as_str(), client)
        }
        WorkerRequest::DeleteSession(url, session_name) => {
            delete_session(url, session_name.as_str(), client)
        }
    }
}

fn check_server_connection(url_opt: Option<Url>, client: &Client) -> WorkerEvent {
    let Some(url) = url_opt else {
        return WorkerEvent::ServerHealth { ok: false };
    };
    let check_url = match url.join("check") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::ServerHealth { ok: false };
        }
    };

    match client.get(check_url).send() {
        Ok(resp) => {
            if let Ok(resp_text) = resp.text() {
                if resp_text == "PMSU" {
                    WorkerEvent::ServerHealth { ok: true }
                } else {
                    WorkerEvent::ServerHealth { ok: false }
                }
            } else {
                WorkerEvent::ServerHealth { ok: false }
            }
        }
        Err(_) => WorkerEvent::ServerHealth { ok: false },
    }
}

fn get_sessions(url: Url, client: &Client) -> WorkerEvent {
    let sessions_url = match url.join("sessions") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from(
                "Error polling for sessions. Could not set url",
            ));
        }
    };

    match client.get(sessions_url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                let msg = format!(
                    "Error polling for sessions. Server returned status {}",
                    resp.status()
                );
                log::warn!("{msg}");
                return WorkerEvent::Error(msg);
            }

            match resp.text() {
                Ok(body) => parse_get_sessions(body.as_str()),
                Err(e) => {
                    log::warn!("Error reading sessions response body: {e}");
                    WorkerEvent::Error(
                        "Error polling for sessions. Could not read server response".to_string(),
                    )
                }
            }
        }
        Err(e) => {
            log::warn!("Got an erroneous response from server while fetching sessions. Error {e}");
            WorkerEvent::Error(String::from(
                "Error polling for sessions. Could not get sessions list",
            ))
        }
    }
}

fn create_session(url: Url, session_name: &str, client: &Client) -> WorkerEvent {
    let sessions_url = match url.join("sessions") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error creating session. Could not set url"));
        }
    };

    #[derive(serde::Serialize)]
    struct SessionPost<'a> {
        session: &'a str,
    }

    let body = SessionPost {
        session: session_name,
    };

    match client.post(sessions_url).json(&body).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                WorkerEvent::CreatedSession(session_name.to_string())
            } else {
                let msg = if status == reqwest::StatusCode::CONFLICT {
                    format!("Error creating session. Session '{session_name}' already exists")
                } else if status == reqwest::StatusCode::NOT_FOUND {
                    "Error creating session. Server endpoint not found".to_string()
                } else {
                    format!("Error creating session. Server returned status {status}")
                };
                log::warn!("{msg}");
                WorkerEvent::Error(msg)
            }
        }
        Err(e) => {
            log::warn!("Error sending create session request: {e}");
            WorkerEvent::Error(String::from(
                "Error creating session. Could not reach server",
            ))
        }
    }
}

fn delete_session(url: Url, session_name: &str, client: &Client) -> WorkerEvent {
    let sessions_url = match url.join("sessions") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error deleting session. Could not set url"));
        }
    };

    #[derive(serde::Serialize)]
    struct SessionPost<'a> {
        session: &'a str,
    }

    let body = SessionPost {
        session: session_name,
    };

    match client.delete(sessions_url).json(&body).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                WorkerEvent::DeletedSession
            } else {
                let msg = if status == reqwest::StatusCode::NOT_FOUND {
                    format!("Error deleting session. Session '{session_name}' not found")
                } else {
                    format!("Error deleting session. Server returned status {status}")
                };
                log::warn!("{msg}");
                WorkerEvent::Error(msg)
            }
        }
        Err(e) => {
            log::warn!("Error sending delete session request: {e}");
            WorkerEvent::Error(String::from(
                "Error deleting session. Could not reach server",
            ))
        }
    }
}