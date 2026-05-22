use anyhow::{Context, Result};
use directories::BaseDirs;
use holla_proto::{Config, Message};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

pub fn messages_path() -> Result<PathBuf> {
    let dirs = BaseDirs::new().context("could not find a valid home directory")?;

    Ok(dirs.home_dir().join(".holla").join("messages.jsonl"))
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = BaseDirs::new().context("could not find a valid home directory")?;

    Ok(dirs.home_dir().join(".holla").join("config.json"))
}

pub fn init_config() -> Result<PathBuf> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create config directory: {}", parent.display()))?;
    }

    if !path.exists() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config)?;
        fs::write(&path, json)
            .with_context(|| format!("could not write config file: {}", path.display()))?;
    }

    Ok(path)
}

pub fn append_message(message: &Message) -> Result<()> {
    let path = messages_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create data directory: {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("could not open messages file: {}", path.display()))?;

    let line = serde_json::to_string(message)?;
    writeln!(file, "{line}")?;

    Ok(())
}

pub fn read_messages() -> Result<Vec<Message>> {
    let path = messages_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .with_context(|| format!("could not open messages file: {}", path.display()))?;

    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = line?;

        let message: Message = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(error) => {
                eprintln!("Skipping unreadable message: {error}");
                continue;
            }
        };

        messages.push(message);
    }

    Ok(messages)
}

pub fn read_config() -> Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        init_config()?;
    }

    let json = fs::read_to_string(&path)
        .with_context(|| format!("could not read config file: {}", path.display()))?;

    let config = serde_json::from_str(&json)
        .with_context(|| format!("could not parse config file: {}", path.display()))?;

    Ok(config)
}
