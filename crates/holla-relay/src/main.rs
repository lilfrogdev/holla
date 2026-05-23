use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use holla_proto::{Message, normalize_room};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Clone)]
struct AppState {
    messages: Arc<Mutex<Vec<Message>>>,
}

#[derive(Debug, Deserialize)]
struct ListMessagesQuery {
    room: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("holla_relay=info".parse()?))
        .init();

    let state = AppState {
        messages: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/messages", get(list_messages).post(create_message))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:46552").await?;
    info!("holla-relay listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

async fn list_messages(
    State(state): State<AppState>,
    Query(query): Query<ListMessagesQuery>,
) -> Json<Vec<Message>> {
    let room_filter = query.room.map(normalize_room);

    let messages = state.messages.lock().expect("messages mutex poisoned");

    let filtered = messages
        .iter()
        .filter(|message| match room_filter.as_deref() {
            Some(room) => message.room == room,
            None => true,
        })
        .cloned()
        .collect();

    Json(filtered)
}

async fn create_message(
    State(state): State<AppState>,
    Json(mut message): Json<Message>,
) -> Json<Message> {
    message.room = normalize_room(&message.room);

    let mut messages = state.messages.lock().expect("messages mutex poisoned");
    messages.push(message.clone());

    Json(message)
}
