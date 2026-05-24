use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sent_at: DateTime<Utc>,
    pub workspace: String,
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

pub fn normalize_room(room: impl AsRef<str>) -> String {
    room.as_ref().trim().trim_start_matches('#').to_string()
}

impl Message {
    pub fn new(
        workspace: impl AsRef<str>,
        room: impl AsRef<str>,
        sender: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sent_at: Utc::now(),
            workspace: normalize_workspace(workspace),
            room: normalize_room(room),
            sender: sender.into(),
            body: body.into(),
        }
    }
}

pub fn normalize_workspace(workspace: impl AsRef<str>) -> String {
    let workspace = workspace.as_ref().trim();

    if workspace.is_empty() {
        "default".to_string()
    } else {
        workspace.to_string()
    }
}
