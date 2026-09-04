mod handlers;

use std::{collections::HashMap, sync::Arc};

use clap::Parser;

use axum::{
    Router,
    routing::{get, post},
};
use handlers::{
    delete_sessions, finish_item, get_item_info, get_items, get_sessions, post_items,
    post_sessions, reveal_item, vote_for_item,
};
use poker_shared::PokerSession;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Config {
    /// Optional hostname, default 0:0:0:0
    #[arg(short('b'), long, default_value = "0.0.0.0")]
    host: String,

    /// Port, defaults to 8080
    #[arg(short, long, default_value_t = 8080)]
    port: usize,
}

pub struct PokerState {
    sessions: HashMap<String, PokerSession>,
}

#[derive(Clone)]
pub struct AppState {
    poker: Arc<Mutex<PokerState>>,
}

#[tokio::main]
async fn main() {
    colog::init();

    let config = Config::parse();

    log::info!("Starting pokerkees server, reading arguments");
    log::info!(
        "Read config -> host: {}, port: {}",
        config.host,
        config.port
    );

    // Create the webservice application
    let app = create_app();

    log::info!("Created app, starting server");

    // bind a TcpListener, using config, and serve
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn create_app() -> axum::Router {
    let p_state = PokerState {
        sessions: HashMap::new(),
    };
    let state = AppState {
        poker: Arc::new(Mutex::new(p_state)),
    };

    Router::new()
        .route("/check", get(|| async { "PMSU" }))
        .route(
            "/sessions",
            get(get_sessions)
                .post(post_sessions)
                .delete(delete_sessions),
        )
        .route("/items", get(get_items).post(post_items))
        .route("/item_info", get(get_item_info))
        .route("/vote", post(vote_for_item))
        .route("/reveal", post(reveal_item))
        .route("/finish", post(finish_item))
        .with_state(state)
}
