mod store;

use anyhow::Result;
use clap::{Parser, Subcommand};
use holla_proto::{Message, normalize_room};

#[derive(Debug, Parser)]
#[command(name = "holla")]
#[command(about = "Encrypted async chat for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,

    Send {
        room: String,
        body: String,

        #[arg(long)]
        relay: Option<String>,
    },

    Recv {
        #[arg(long)]
        room: Option<String>,

        #[arg(long)]
        json: bool,
    },

    Debug {
        #[command(subcommand)]
        command: DebugCommand,
    },
}

#[derive(Debug, Subcommand)]
enum DebugCommand {
    Paths,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            let path = store::init_config()?;
            println!("Initialized holla config: {}", path.display());
        }
        Command::Send { room, body, relay } => {
            let config = store::read_config()?;
            let message = Message::new(room, config.default_sender, body);

            store::append_message(&message)?;
            println!("{}", serde_json::to_string_pretty(&message)?);

            if let Some(relay) = relay {
                send_to_relay(&relay, &message).await?;
            }
        }
        Command::Recv { room, json } => {
            recv_messages(room.as_deref(), json)?;
        }
        Command::Debug { command } => match command {
            DebugCommand::Paths => {
                print_paths()?;
            }
        },
    }

    Ok(())
}

fn print_paths() -> Result<()> {
    println!("config:   {}", store::config_path()?.display());
    println!("messages: {}", store::messages_path()?.display());

    Ok(())
}

fn recv_messages(room_filter: Option<&str>, json: bool) -> Result<()> {
    let room_filter = room_filter.map(normalize_room);
    let messages = store::read_messages()?;

    if messages.is_empty() {
        println!("No messages yet.");
        return Ok(());
    }

    for message in messages {
        if let Some(room_filter) = room_filter.as_deref()
            && message.room != room_filter
        {
            continue;
        }

        if json {
            println!("{}", serde_json::to_string(&message)?);
        } else {
            println!(
                "[{}] [#{}] {}: {}",
                message.sent_at, message.room, message.sender, message.body
            );
        }
    }

    Ok(())
}

async fn send_to_relay(relay: &str, message: &Message) -> Result<()> {
    let url = format!("{}/messages", relay.trim_end_matches('/'));

    let client = reqwest::Client::new();
    client
        .post(url)
        .json(message)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
