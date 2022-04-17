use std::env;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, Configuration, StandardFramework,
};
use serenity::model::channel::Message;

const DEFAULT_COMMAND_PREFIX: &str = "/";
const COMMAND_PREFIX_ENVVAR: &str = "STARFORGED_COMMAND_PREFIX";
const TOKEN_ENVVAR: &str = "STARFORGED_DISCORD_TOKEN";
const MISSING_TOKEN_ERROR: &str = "Missing STARFORGED_DISCORD_TOKEN environment variable";

/// The group of all our commands.
#[group]
#[commands(ping)]
struct Commands;

/// Our request handler.
struct Handler;

#[async_trait]
impl EventHandler for Handler {}

fn framework_config(config: &mut Configuration) -> &mut Configuration {
    let prefix =
        env::var(COMMAND_PREFIX_ENVVAR).unwrap_or_else(|_| DEFAULT_COMMAND_PREFIX.to_string());
    config.prefix(prefix)
}

#[tokio::main]
async fn main() {
    // Create our framework, specifying the command prefix and commands.
    let framework = StandardFramework::new()
        .configure(framework_config)
        .group(&COMMANDS_GROUP); // This constant is derived by #[group].

    // Create our client and log in.
    let token = env::var(TOKEN_ENVVAR).expect(MISSING_TOKEN_ERROR);
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Enter main command loop.
    if let Err(e) = client.start().await {
        eprintln!("Error: {:?}", e);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
