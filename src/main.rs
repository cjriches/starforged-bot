use std::env;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, Configuration, StandardFramework,
};
use serenity::model::channel::Message;

use crate::rolls::{ActionRoll, CustomRoll, OracleRoll, ProgressRoll, RollSpec};

mod rolls;

const DEFAULT_COMMAND_PREFIX: &str = "/";
const COMMAND_PREFIX_ENVVAR: &str = "STARFORGED_COMMAND_PREFIX";
const TOKEN_ENVVAR: &str = "STARFORGED_DISCORD_TOKEN";
const MISSING_TOKEN_ERROR: &str = "Missing STARFORGED_DISCORD_TOKEN environment variable";

/// The group of all our commands.
#[group]
#[commands(ping, action_roll, progress_roll, oracle_roll, custom_roll)]
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

/// A macro for sending a message.
macro_rules! send {
    ($ctx:expr, $msg:expr, $content:expr) => {
        $msg.channel_id.send_message($ctx, |m| m.content($content))
    };
}

/// Simple ping command to check the bot is online.
#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

/// Perform an action roll.
#[command]
#[aliases("move", "action", "ar", "a")]
async fn action_roll(ctx: &Context, msg: &Message) -> CommandResult {
    // Parse the roll.
    let args = msg.content.split_whitespace().skip(1).collect::<Vec<_>>();
    let bonus = if args.is_empty() {
        None
    } else {
        let mut bonus = 0;
        for arg in args {
            let val = arg.parse::<u32>();
            match val {
                Ok(v) => bonus += v,
                Err(_) => {
                    let response = format!("Invalid bonus: {}", arg);
                    msg.reply(ctx, response).await?;
                    return Ok(());
                }
            }
        }
        Some(bonus)
    };

    // Make the roll.
    let roll = ActionRoll::random(bonus);
    let response = roll.to_string();

    // Delete the message and respond to it.
    msg.delete(ctx).await?;
    send!(ctx, msg, response).await?;

    Ok(())
}

/// Perform a progress roll.
#[command]
#[aliases("progress", "pr", "p")]
async fn progress_roll(ctx: &Context, msg: &Message) -> CommandResult {
    // Parse the roll.
    let args = msg.content.split_whitespace().skip(1).collect::<Vec<_>>();
    let bonus = match args.len() {
        0 => None,
        1 => {
            let bonus = args[0].parse::<u32>();
            if bonus.is_err() {
                let response = format!("Invalid progress: {}", args[0]);
                msg.reply(ctx, response).await?;
                return Ok(());
            }
            Some(bonus.unwrap())
        }
        n => {
            let response = format!("Too many arguments (expected 0 or 1, got {})", n);
            msg.reply(ctx, response).await?;
            return Ok(());
        }
    };

    // Make the roll.
    let roll = ProgressRoll::random(bonus);
    let response = roll.to_string();

    // Delete the message and respond to it.
    msg.delete(ctx).await?;
    send!(ctx, msg, response).await?;

    Ok(())
}

/// Perform an oracle roll.
#[command]
#[aliases("oracle", "or", "o")]
async fn oracle_roll(ctx: &Context, msg: &Message) -> CommandResult {
    // Parse the roll.
    let num_args = msg.content.split_whitespace().skip(1).count();
    if num_args > 0 {
        let response = format!("Too many arguments (expected 0, got {})", num_args);
        msg.reply(ctx, response).await?;
        return Ok(());
    }

    // Make the roll.
    let roll = OracleRoll::random();
    let response = roll.to_string();

    // Delete the message and respond to it.
    msg.delete(ctx).await?;
    send!(ctx, msg, response).await?;

    Ok(())
}

/// Perform a custom roll.
#[command]
#[aliases("roll", "r")]
async fn custom_roll(ctx: &Context, msg: &Message) -> CommandResult {
    // Parse the roll.
    let mut specs: Vec<RollSpec> = Vec::new();
    for arg in msg.content.split_whitespace().skip(1) {
        let spec = arg.parse();
        if let Ok(spec) = spec {
            specs.push(spec);
        } else {
            let response = format!("Invalid argument: {}", arg);
            msg.reply(ctx, response).await?;
            return Ok(());
        }
    }
    if specs.is_empty() {
        let response = "Not enough arguments (expected 1+, got 0)";
        msg.reply(ctx, response).await?;
        return Ok(());
    }

    // Make the roll.
    let roll = CustomRoll::random(specs);
    let response = roll.to_string();

    // Delete the message and respond to it.
    msg.delete(ctx).await?;
    send!(ctx, msg, response).await?;

    Ok(())
}
