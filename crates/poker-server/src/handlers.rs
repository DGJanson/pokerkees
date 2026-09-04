use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use poker_shared::{PokerItem, PokerSession};

use crate::AppState;

// !!!!!!!!!!!!!!!!!!!!!!!!!!!!
// !! -- SESSION HANDLERS -- !!

#[derive(serde::Serialize)]
pub struct SessionsResponse {
    sessions: Vec<String>,
}

pub async fn get_sessions(State(state): State<AppState>) -> Json<SessionsResponse> {
    let sessions = {
        let poker = state.poker.lock().await;
        // simply return a list of sessions:
        poker.sessions.keys().cloned().collect::<Vec<String>>()
    };

    Json(SessionsResponse { sessions })
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SessionPost {
    session: String,
}

#[allow(clippy::map_entry)]
pub async fn post_sessions(
    State(state): State<AppState>,
    Json(session): Json<SessionPost>,
) -> StatusCode {
    let mut poker = state.poker.lock().await;
    let SessionPost { session } = session;

    if poker.sessions.contains_key(&session) {
        StatusCode::CONFLICT
    } else {
        log::info!("Creating session {session}");
        poker.sessions.insert(
            session,
            PokerSession {
                started: chrono::Utc::now(),
                items: HashMap::new(),
            },
        );
        StatusCode::CREATED
    }
}

pub async fn delete_sessions(
    State(state): State<AppState>,
    Json(session): Json<SessionPost>,
) -> StatusCode {
    let mut poker = state.poker.lock().await;
    let SessionPost { session } = session;

    if poker.sessions.remove(&session).is_some() {
        log::info!("Removed session {session}");
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

// !!!!!!!!!!!!!!!!!!!!!!!!!!!
// !! --  ITEM HANDLERS  -- !!

#[derive(serde::Deserialize)]
pub struct GetItemsQuery {
    session: String,
}

#[derive(serde::Serialize)]
pub struct ItemInfo {
    id: usize,
    revealed: bool,
    done: bool,
}

#[derive(serde::Serialize)]
pub struct ItemsResponse {
    items: Vec<ItemInfo>,
}

// Check if sessions exists, if not return NOT_FOUND
// If found, return all available items with
// - item nr (id)
// - revelead yes / no
// - finished yes / no
pub async fn get_items(
    State(state): State<AppState>,
    Query(query): Query<GetItemsQuery>,
) -> Result<Json<ItemsResponse>, StatusCode> {
    let poker = state.poker.lock().await;
    let Some(session) = poker.sessions.get(&query.session) else {
        return Err(StatusCode::NOT_FOUND);
    };

    let items = session
        .items
        .iter()
        .map(|(id, item)| ItemInfo {
            id: *id,
            revealed: item.revealed,
            done: item.done,
        })
        .collect::<Vec<_>>();

    // Note this returns a 200 response
    Ok(Json(ItemsResponse { items }))
}

#[derive(serde::Deserialize)]
pub struct PostItem {
    session: String,
    item_id: usize,
    item_title: Option<String>,
}

// Check if the session exists AND if the item already exists
pub async fn post_items(State(state): State<AppState>, Json(req): Json<PostItem>) -> StatusCode {
    let mut poker = state.poker.lock().await;
    let Some(session) = poker.sessions.get_mut(&req.session) else {
        return StatusCode::NOT_FOUND;
    };

    if session.items.contains_key(&req.item_id) {
        return StatusCode::CONFLICT;
    }

    session.items.insert(
        req.item_id,
        PokerItem {
            item_id: req.item_id,
            item_title: req.item_title.unwrap_or_default(),
            votes: HashMap::new(),
            revealed: false,
            done: false,
        },
    );

    StatusCode::CREATED
}

#[derive(serde::Deserialize)]
pub struct ItemInfoQuery {
    session: String,
    item_id: usize,
}

pub async fn get_item_info(
    State(state): State<AppState>,
    Query(query): Query<ItemInfoQuery>,
) -> Result<Json<PokerItem>, StatusCode> {
    let poker = state.poker.lock().await;
    // look for session
    let Some(session) = poker.sessions.get(&query.session) else {
        return Err(StatusCode::NOT_FOUND);
    };

    let Some(item) = session.items.get(&query.item_id) else {
        return Err(StatusCode::NOT_FOUND);
    };

    Ok(Json(item.clone()))
}

// We can reuse the ItemInfoQuery  struct here
pub async fn reveal_item(
    State(state): State<AppState>,
    Json(query): Json<ItemInfoQuery>,
) -> StatusCode {
    let mut poker = state.poker.lock().await;
    // look for session
    let Some(session) = poker.sessions.get_mut(&query.session) else {
        return StatusCode::NOT_FOUND;
    };

    let Some(item) = session.items.get_mut(&query.item_id) else {
        return StatusCode::NOT_FOUND;
    };

    item.revealed = true;

    StatusCode::OK
}

// We can reuse the ItemInfoQuery  struct here
pub async fn finish_item(
    State(state): State<AppState>,
    Json(query): Json<ItemInfoQuery>,
) -> StatusCode {
    let mut poker = state.poker.lock().await;
    // look for session
    let Some(session) = poker.sessions.get_mut(&query.session) else {
        return StatusCode::NOT_FOUND;
    };

    let Some(item) = session.items.get_mut(&query.item_id) else {
        return StatusCode::NOT_FOUND;
    };

    item.done = true;

    StatusCode::OK
}

#[derive(serde::Deserialize)]
pub struct VotePost {
    session: String,
    item_id: usize,
    user: String,
    vote: usize,
}

pub async fn vote_for_item(
    State(state): State<AppState>,
    Json(vote): Json<VotePost>,
) -> StatusCode {
    let mut poker = state.poker.lock().await;
    // look for session
    let Some(session) = poker.sessions.get_mut(&vote.session) else {
        return StatusCode::NOT_FOUND;
    };

    // get item
    let Some(item) = session.items.get_mut(&vote.item_id) else {
        return StatusCode::NOT_FOUND;
    };

    // Check if valid vote
    if vote.user.is_empty() || vote.vote < 1 || vote.vote > 40 {
        return StatusCode::BAD_REQUEST;
    }

    // Now simply update the vote, regardless of if user
    // already voted
    item.votes.insert(vote.user, vote.vote);

    StatusCode::OK
}
