// Code for the IO worker
// This is a single thread on which will perform IO
// in the background.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use url::Url;

// TODO, clean this
// this is raw LLM output

// UI -> Worker
#[derive(Debug, Clone)]
pub enum WorkerRequest {
    /// Check connection with server, using String parm as url
    CheckServerConnection(Url),
}

// Worker -> UI
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    ServerHealth { ok: bool },
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
    }
}

fn check_server_connection(url: Url, client: &Client) -> WorkerEvent {
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
