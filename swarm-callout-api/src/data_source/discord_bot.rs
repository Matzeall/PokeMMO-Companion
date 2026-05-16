use crate::data_source::{DataSource, SwarmData};
use chrono::{TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use dotenvy::dotenv;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::sync::mpsc::{Receiver, Sender};

struct Config {
    discord_token: String,
    channel_id: u64,
}

fn load_config() -> Config {
    // loads .env file into program environment
    dotenv().expect(".env file not found");

    Config {
        discord_token: env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"),
        channel_id: env::var("ALERT_CHANNEL_ID")
            .expect("ALERT_CHANNEL_ID must be set")
            .parse()
            .expect("ALERT_CHANNEL_ID must be a number"),
    }
}

pub struct DiscordBot {
    data_rx: Option<Receiver<SwarmData>>,
    channel_id: u64,
    discord_token: String,
    current_data: SwarmData,
}

impl DiscordBot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let config = load_config();
        Self {
            data_rx: None,
            channel_id: config.channel_id,
            discord_token: config.discord_token,
            current_data: SwarmData::default(),
        }
    }
}

impl DataSource for DiscordBot {
    fn spawn(&mut self, async_handle: tokio::runtime::Handle) -> tokio::task::JoinHandle<()> {
        println!("Bot starting for channel: {}", self.channel_id);

        let (data_tx, data_rx) = std::sync::mpsc::channel::<SwarmData>();
        self.data_rx = Some(data_rx);

        async_handle.spawn(spawn_message_handler(
            self.discord_token.clone(),
            self.channel_id,
            data_tx,
        ))
    }

    fn get_current_data(&mut self) -> super::SwarmData {
        if let Some(rx) = &mut self.data_rx {
            while let Ok(data) = rx.try_recv() {
                self.current_data = data;
            }
        }

        self.current_data.clone()
    }

    fn is_reliable(&self) -> bool {
        // TODO: check based on last_detected_timestamp etc. when there are multiple sources at
        // some point
        true
    }
}

async fn spawn_message_handler(discord_token: String, channel_id: u64, data_tx: Sender<SwarmData>) {
    let bot_event_handler = MessageHandler {
        channel_id,
        data_tx,
    };

    let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT // read message-content
            | GatewayIntents::GUILD_PRESENCES // online status
            | GatewayIntents::GUILD_MEMBERS; // list members

    let mut client = Client::builder(discord_token, intents)
        .event_handler(bot_event_handler)
        .await
        .expect("Error creating serenity client (discord bot)");

    // starts event loop
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

pub struct MessageHandler {
    channel_id: u64,
    data_tx: Sender<SwarmData>,
}

#[async_trait]
impl EventHandler for MessageHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        // prevent the bot from responding to itself or react to anything outside it's channel_id
        if msg.channel_id != self.channel_id
            || (msg.author.bot && msg.author.id == _ctx.cache.current_user().id)
        {
            return;
        }

        println!("message received:: {} -> {}", msg.author, msg.content);

        if msg.content.contains("@Alpha Swarms") {
            println!(
                "-----------------------------------------------------------------------------------\nalpha alert received ({}): {}\nmessage_reference: {:?}\referenced_message: {:?}\nembed: {:?}",
                msg.timestamp.with_timezone(&Berlin),
                msg.content,
                msg.message_reference,
                msg.referenced_message,
                msg.embeds,
            );

            if msg.embeds.is_empty() {
                let _ = self.data_tx.send(parse_swarm_data(msg));
            }

            // TODO: better error handling? maybe mark this datasource temporarily as not reliable
            // on weird requests, so a lower one may make a better guess
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!(
            "{} is ready and listening to alert messages",
            ready.user.name
        );
    }
}

fn parse_swarm_data(msg: Message) -> SwarmData {
    let mut pokemon = String::new();
    let mut location = String::new();
    let mut region = String::new();
    let mut details = String::new();

    for embed in &msg.embeds {
        for field in &embed.fields {
            println!("recognized embed-field: {} -> {}", field.name, field.value);
            match field.name.trim().to_lowercase().as_str() {
                "monster name" => pokemon = field.value.clone(),
                "location" => location = field.value.clone(),
                "region" => region = field.value.clone(),
                "additional details" => details = field.value.clone(),
                _ => (),
            };
        }
    }

    SwarmData {
        pokemon,
        location,
        region,
        details,
    }
}
