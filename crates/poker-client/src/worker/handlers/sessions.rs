use serde_json::Value;

use crate::worker::WorkerEvent;

// Code for parsing info on sessions
pub fn parse_get_sessions(body: &str) -> WorkerEvent {
    // Try parsing as a raw array of strings first
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Array(arr)) => {
            let mut sessions = Vec::with_capacity(arr.len());
            for v in arr {
                if let Value::String(s) = v {
                    sessions.push(s);
                } else {
                    let msg = "Error parsing sessions: array must contain only strings".to_string();
                    log::warn!("{msg}. Body: {body}");
                    return WorkerEvent::Error(msg);
                }
            }
            WorkerEvent::Sessions { sessions }
        }
        Ok(Value::Object(map)) => {
            // Try extracting "sessions" field from object
            match map.get("sessions") {
                Some(Value::Array(arr)) => {
                    let mut sessions = Vec::with_capacity(arr.len());
                    for v in arr {
                        if let Value::String(s) = v {
                            sessions.push(String::from(s));
                        } else {
                            let msg = "Error parsing sessions: 'sessions' array must contain only strings".to_string();
                            log::warn!("{msg}. Body: {body}");
                            return WorkerEvent::Error(msg);
                        }
                    }
                    WorkerEvent::Sessions { sessions }
                }
                Some(_) => {
                    let msg =
                        "Error parsing sessions: 'sessions' field is not an array".to_string();
                    log::warn!("{msg}. Body: {body}");
                    WorkerEvent::Error(msg)
                }
                None => {
                    let msg = "Error parsing sessions: missing 'sessions' field".to_string();
                    log::warn!("{msg}. Body: {body}");
                    WorkerEvent::Error(msg)
                }
            }
        }
        Ok(_) => {
            let msg = "Error parsing sessions: unexpected JSON structure".to_string();
            log::warn!("{msg}. Body: {body}");
            WorkerEvent::Error(msg)
        }
        Err(e) => {
            let msg = format!("Error parsing sessions JSON: {e}");
            log::warn!("{msg}. Body: {body}");
            WorkerEvent::Error(
                "Error polling for sessions. Could not parse server response".to_string(),
            )
        }
    }
}
