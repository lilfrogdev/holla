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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub remote: Option<String>,
    pub workspace: String,
    pub room: String,
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

pub fn normalize_workspace(workspace: impl AsRef<str>) -> String {
    let workspace = workspace.as_ref().trim();

    if workspace.is_empty() {
        "default".to_string()
    } else {
        workspace.to_string()
    }
}

pub fn parse_destination(input: impl AsRef<str>) -> Destination {
    let input = input.as_ref().trim();
    let parts: Vec<_> = input.split('/').filter(|part| !part.is_empty()).collect();

    match parts.as_slice() {
        [room] => Destination {
            remote: None,
            workspace: "default".to_string(),
            room: normalize_room(room),
        },
        [workspace, room] => Destination {
            remote: None,
            workspace: normalize_workspace(workspace),
            room: normalize_room(room),
        },
        [remote, workspace, room] => Destination {
            remote: Some(remote.trim().to_string()),
            workspace: normalize_workspace(workspace),
            room: normalize_room(room),
        },
        _ => Destination {
            remote: None,
            workspace: "default".to_string(),
            room: normalize_room(input),
        },
    }
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
