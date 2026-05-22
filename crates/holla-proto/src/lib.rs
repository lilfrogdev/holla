use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sent_at: DateTime<Utc>,
    pub room: String,
    pub sender: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_sender: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_sender: "@local#dev".to_string(),
        }
    }
}

impl Message {
    pub fn new(
        room: impl Into<String>,
        sender: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sent_at: Utc::now(),
            room: room.into(),
            sender: sender.into(),
            body: body.into(),
        }
    }
}
