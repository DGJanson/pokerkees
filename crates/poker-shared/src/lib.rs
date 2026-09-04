// This is basically "just" a collection of definitions shared by
// client and server

use std::collections::HashMap;

use chrono::{DateTime, Utc};

pub struct PokerSession {
    /// Start time of the session
    pub started: DateTime<Utc>,
    /// The different items (tasks, user stories) that are estimated
    pub items: HashMap<usize, PokerItem>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct PokerItem {
    /// Reference to an item ID (note it is numeric)
    pub item_id: usize,
    /// Possible title, description of the item
    pub item_title: String,
    /// Hashmap holding the votes, the String is the username
    pub votes: HashMap<String, usize>, // User, vote
    /// Whether the votes are visible for everyone
    pub revealed: bool,
    /// Whether the items has been handled completely
    pub done: bool,
}
