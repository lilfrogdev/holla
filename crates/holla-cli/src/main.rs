mod store;

use anyhow::Result;
use clap::{Parser, Subcommand};
use holla_proto::{Message, normalize_room, normalize_workspace, parse_destination};

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
        destination: String,
        body: String,

        #[arg(long)]
        relay: Option<String>,
    },

    Recv {
        #[arg(long)]
        room: Option<String>,

        #[arg(long, default_value = "default")]
        workspace: String,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        relay: Option<String>,
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
        Command::Send {
            destination,
            body,
            relay,
        } => {
            let destination = parse_destination(destination);
            let config = store::read_config()?;
            let message = Message::new(
                destination.workspace,
                destination.room,
                config.default_sender,
                body,
            );

            store::append_message(&message)?;
            println!("{}", serde_json::to_string_pretty(&message)?);

            if let Some(relay) = relay {
                send_to_relay(&relay, &message).await?;
            }
        }
        Command::Recv {
            room,
            workspace,
            json,
            relay,
        } => {
            recv_messages(&workspace, room.as_deref(), json, relay.as_deref()).await?;
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

async fn recv_messages(
    workspace_filter: &str,
    room_filter: Option<&str>,
    json: bool,
    relay: Option<&str>,
) -> Result<()> {
    let workspace_filter = normalize_workspace(workspace_filter);
    let room_filter = room_filter.map(normalize_room);

    let messages = if let Some(relay) = relay {
        recv_from_relay(relay, &workspace_filter, room_filter.as_deref()).await?
    } else {
        store::read_messages()?
    };

    if messages.is_empty() {
        println!("No messages yet.");
        return Ok(());
    }

    for message in messages {
        if message.workspace != workspace_filter {
            continue;
        }

        if let Some(room_filter) = room_filter.as_deref()
            && message.room != room_filter
        {
            continue;
        }

        if json {
            println!("{}", serde_json::to_string(&message)?);
        } else {
            println!(
                "[{}] [{}#{}] {}: {}",
                message.sent_at, message.workspace, message.room, message.sender, message.body
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

async fn recv_from_relay(
    relay: &str,
    workspace_filter: &str,
    room_filter: Option<&str>,
) -> Result<Vec<Message>> {
    let base = relay.trim_end_matches('/');
    let mut url = format!("{base}/messages?workspace={workspace_filter}");

    if let Some(room) = room_filter {
        url.push_str("&room=");
        url.push_str(room);
    }

    let client = reqwest::Client::new();
    let messages = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Message>>()
        .await?;

    Ok(messages)
}
