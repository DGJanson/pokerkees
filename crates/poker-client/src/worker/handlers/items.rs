use serde_json::Value;

use poker_shared::ItemInfo;

use crate::worker::WorkerEvent;

// Parse response for GET /items
// Accept both a raw array of items or an object containing "items": [...]
pub fn parse_get_items(body: &str) -> WorkerEvent {
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Array(arr)) => match parse_items_array(&arr) {
            Ok(items) => WorkerEvent::Items { items },
            Err(msg) => WorkerEvent::Error(msg),
        },
        Ok(Value::Object(map)) => match map.get("items") {
            Some(Value::Array(arr)) => match parse_items_array(arr) {
                Ok(items) => WorkerEvent::Items { items },
                Err(msg) => WorkerEvent::Error(msg),
            },
            Some(_) => {
                let msg = "Error parsing items: 'items' field is not an array".to_string();
                WorkerEvent::Error(msg)
            }
            None => {
                let msg = "Error parsing items: missing 'items' field".to_string();
                WorkerEvent::Error(msg)
            }
        },
        Ok(_) => {
            let msg = "Error parsing items: unexpected JSON structure".to_string();
            WorkerEvent::Error(msg)
        }
        Err(_) => WorkerEvent::Error(
            "Error polling for items. Could not parse server response".to_string(),
        ),
    }
}

fn parse_items_array(arr: &[Value]) -> Result<Vec<ItemInfo>, String> {
    let mut items = Vec::with_capacity(arr.len());
    for v in arr {
        let Value::Object(map) = v else {
            return Err("Error parsing items: array must contain only objects".to_string());
        };

        let id = match map.get("id") {
            Some(Value::Number(n)) => n
                .as_u64()
                .ok_or_else(|| "Error parsing items: 'id' must be a positive integer".to_string())?
                as usize,
            _ => return Err("Error parsing items: missing or invalid 'id'".to_string()),
        };

        let revealed = match map.get("revealed") {
            Some(Value::Bool(b)) => *b,
            _ => return Err("Error parsing items: missing or invalid 'revealed'".to_string()),
        };

        let done = match map.get("done") {
            Some(Value::Bool(b)) => *b,
            _ => return Err("Error parsing items: missing or invalid 'done'".to_string()),
        };

        let item_title = match map.get("item_title") {
            Some(s) => s.to_string(),
            _ => return Err("Error parsing items: missing or invalild 'item_title'".to_string()),
        };

        items.push(ItemInfo {
            id,
            revealed,
            done,
            item_title,
        });
    }
    Ok(items)
}

// Parse response for GET /item_info
// Expects the raw PokerItem object
pub fn parse_get_item_info(body: &str) -> WorkerEvent {
    match serde_json::from_str::<poker_shared::PokerItem>(body) {
        Ok(item) => WorkerEvent::ItemInfo { item },
        Err(_) => WorkerEvent::Error(
            "Error fetching item info. Could not parse server response".to_string(),
        ),
    }
}
