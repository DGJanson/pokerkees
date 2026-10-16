// Code for the IO worker
// This is a single thread on which will perform IO
// in the background.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use handlers::azure::fetch_azure_item;
use handlers::items::{parse_get_item_info, parse_get_items};
use handlers::sessions::{self, parse_get_sessions};
use poker_shared::{ItemInfo, PokerItem};
use reqwest::blocking::Client;
use url::Url;

use crate::state::AzureWorkItem;

mod handlers;

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

    // Items
    GetItems(Url, String),
    CreateItem(Url, String, usize, Option<String>),
    GetItemInfo(Url, String, usize),
    RevealItem(Url, String, usize),
    FinishItem(Url, String, usize),
    VoteForItem(Url, String, usize, String, usize),

    // Azure stuff
    AzureItem(String, usize),
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

    // Items list for a session
    Items { items: Vec<ItemInfo> },
    // Detailed info of a single item
    ItemInfo { item: PokerItem },
    // Item created
    CreatedItem {},
    // Successful reveal/finish/vote acks
    ItemRevealed,
    ItemFinished,
    ItemVoted,

    // Azure
    AzureItem(AzureWorkItem),

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

        // Items
        WorkerRequest::GetItems(url, session) => get_items(url, session.as_str(), client),
        WorkerRequest::CreateItem(url, session, item_id, item_title) => create_item(
            url,
            session.as_str(),
            item_id,
            item_title.as_deref(),
            client,
        ),
        WorkerRequest::GetItemInfo(url, session, item_id) => {
            get_item_info(url, session.as_str(), item_id, client)
        }
        WorkerRequest::RevealItem(url, session, item_id) => {
            reveal_item(url, session.as_str(), item_id, client)
        }
        WorkerRequest::FinishItem(url, session, item_id) => {
            finish_item(url, session.as_str(), item_id, client)
        }
        WorkerRequest::VoteForItem(url, session, item_id, user, vote) => {
            vote_for_item(url, session.as_str(), item_id, user.as_str(), vote, client)
        }

        // Azure stuff
        WorkerRequest::AzureItem(org, item_id) => get_azure_cli_info(org, item_id),
    }
}

// Functions used by the worker
// Note that errors should be returned as an event, not logged please.
// (It will mess with the interface if we log)
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
                return WorkerEvent::Error(msg);
            }

            match resp.text() {
                Ok(body) => parse_get_sessions(body.as_str()),
                Err(_) => WorkerEvent::Error(
                    "Error polling for sessions. Could not read server response".to_string(),
                ),
            }
        }
        Err(_) => WorkerEvent::Error(String::from(
            "Error polling for sessions. Could not get sessions list",
        )),
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
                WorkerEvent::Error(msg)
            }
        }
        Err(_) => WorkerEvent::Error(String::from(
            "Error creating session. Could not reach server",
        )),
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
                WorkerEvent::Error(msg)
            }
        }
        Err(_) => WorkerEvent::Error(String::from(
            "Error deleting session. Could not reach server",
        )),
    }
}

fn get_items(url: Url, session: &str, client: &Client) -> WorkerEvent {
    let items_url = match url.join("items") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error polling for items. Could not set url"));
        }
    };

    match client.get(items_url).query(&[("session", session)]).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let msg = if status == reqwest::StatusCode::NOT_FOUND {
                    format!("Error polling for items. Session '{session}' not found")
                } else {
                    format!("Error polling for items. Server returned status {status}")
                };
                return WorkerEvent::Error(msg);
            }

            match resp.text() {
                Ok(body) => parse_get_items(body.as_str()),
                Err(_) => WorkerEvent::Error(
                    "Error polling for items. Could not read server response".to_string(),
                ),
            }
        }
        Err(_) => WorkerEvent::Error(String::from(
            "Error polling for items. Could not get items list",
        )),
    }
}

fn create_item(
    url: Url,
    session: &str,
    item_id: usize,
    item_title: Option<&str>,
    client: &Client,
) -> WorkerEvent {
    let items_url = match url.join("items") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error creating item. Could not set url"));
        }
    };

    #[derive(serde::Serialize)]
    struct PostItem<'a> {
        session: &'a str,
        item_id: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        item_title: Option<&'a str>,
    }

    let body = PostItem {
        session,
        item_id,
        item_title,
    };

    match client.post(items_url).json(&body).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                WorkerEvent::CreatedItem {}
            } else {
                let msg = if status == reqwest::StatusCode::NOT_FOUND {
                    format!("Error creating item. Session '{session}' not found")
                } else if status == reqwest::StatusCode::CONFLICT {
                    format!(
                        "Error creating item. Item with id {item_id} already exists in session '{session}'"
                    )
                } else {
                    format!("Error creating item. Server returned status {status}")
                };
                WorkerEvent::Error(msg)
            }
        }
        Err(_) => WorkerEvent::Error(String::from("Error creating item. Could not reach server")),
    }
}

fn get_item_info(url: Url, session: &str, item_id: usize, client: &Client) -> WorkerEvent {
    let info_url = match url.join("item_info") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error fetching item info. Could not set url"));
        }
    };

    match client
        .get(info_url)
        .query(&[("session", session), ("item_id", &item_id.to_string())])
        .send()
    {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let msg = if status == reqwest::StatusCode::NOT_FOUND {
                    String::from("Error fetching item info. Session or item not found")
                } else {
                    format!("Error fetching item info. Server returned status {status}")
                };
                return WorkerEvent::Error(msg);
            }

            match resp.text() {
                Ok(body) => parse_get_item_info(body.as_str()),
                Err(_) => WorkerEvent::Error(
                    "Error fetching item info. Could not read server response".to_string(),
                ),
            }
        }
        Err(_) => WorkerEvent::Error(String::from(
            "Error fetching item info. Could not reach server",
        )),
    }
}

fn reveal_item(url: Url, session: &str, item_id: usize, client: &Client) -> WorkerEvent {
    let reveal_url = match url.join("reveal") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error revealing item. Could not set url"));
        }
    };

    #[derive(serde::Serialize)]
    struct ItemInfoQuery<'a> {
        session: &'a str,
        item_id: usize,
    }

    let body = ItemInfoQuery { session, item_id };

    match client.post(reveal_url).json(&body).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                WorkerEvent::ItemRevealed
            } else {
                let msg = if status == reqwest::StatusCode::NOT_FOUND {
                    "Error revealing item. Session or item not found".to_string()
                } else {
                    format!("Error revealing item. Server returned status {status}")
                };
                WorkerEvent::Error(msg)
            }
        }
        Err(_) => WorkerEvent::Error(String::from("Error revealing item. Could not reach server")),
    }
}

fn finish_item(url: Url, session: &str, item_id: usize, client: &Client) -> WorkerEvent {
    let finish_url = match url.join("finish") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error finishing item. Could not set url"));
        }
    };

    #[derive(serde::Serialize)]
    struct ItemInfoQuery<'a> {
        session: &'a str,
        item_id: usize,
    }

    let body = ItemInfoQuery { session, item_id };

    match client.post(finish_url).json(&body).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                WorkerEvent::ItemFinished
            } else {
                let msg = if status == reqwest::StatusCode::NOT_FOUND {
                    "Error finishing item. Session or item not found".to_string()
                } else {
                    format!("Error finishing item. Server returned status {status}")
                };
                WorkerEvent::Error(msg)
            }
        }
        Err(_) => WorkerEvent::Error(String::from("Error finishing item. Could not reach server")),
    }
}

fn vote_for_item(
    url: Url,
    session: &str,
    item_id: usize,
    user: &str,
    vote: usize,
    client: &Client,
) -> WorkerEvent {
    let vote_url = match url.join("vote") {
        Ok(u) => u,
        Err(_) => {
            return WorkerEvent::Error(String::from("Error voting for item. Could not set url"));
        }
    };

    #[derive(serde::Serialize)]
    struct VotePost<'a> {
        session: &'a str,
        item_id: usize,
        user: &'a str,
        vote: usize,
    }

    let body = VotePost {
        session,
        item_id,
        user,
        vote,
    };

    match client.post(vote_url).json(&body).send() {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                WorkerEvent::ItemVoted
            } else {
                let msg = if status == reqwest::StatusCode::BAD_REQUEST {
                    "Error voting for item. Invalid vote".to_string()
                } else if status == reqwest::StatusCode::NOT_FOUND {
                    "Error voting for item. Session or item not found".to_string()
                } else {
                    format!("Error voting for item. Server returned status {status}")
                };
                WorkerEvent::Error(msg)
            }
        }
        Err(_) => WorkerEvent::Error(String::from(
            "Error voting for item. Could not reach server",
        )),
    }
}

// Azure items
fn get_azure_cli_info(org: String, item_id: usize) -> WorkerEvent {
    match fetch_azure_item(&org, item_id) {
        Ok(awi) => WorkerEvent::AzureItem(awi),
        Err(e) => WorkerEvent::Error(e),
    }
}
