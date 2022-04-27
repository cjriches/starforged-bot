use std::env;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, Configuration, StandardFramework,
};
use serenity::model::channel::Message;

use crate::rolls::{ActionRoll, CustomRoll, OracleRoll, ProgressRoll};

mod parse_roll_spec;
mod rolls;

/// The numeric type used when parsing inputs.
type InputType = u8;
/// The numeric type used for intermediate computations and outputs.
type OutputType = u32;

const DEFAULT_COMMAND_PREFIX: &str = "!";
const COMMAND_PREFIX_ENVVAR: &str = "STARFORGED_COMMAND_PREFIX";
const TOKEN_ENVVAR: &str = "STARFORGED_DISCORD_TOKEN";
const MISSING_TOKEN_ERROR: &str = "Missing STARFORGED_DISCORD_TOKEN environment variable";

/// The group of all our commands.
#[group]
#[commands(ping, help, action_roll, progress_roll, oracle_roll, custom_roll)]
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
            let val = arg.parse::<InputType>();
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
            let bonus = args[0].parse::<InputType>();
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
    let args = msg.content.split_whitespace().skip(1).collect::<Vec<_>>();
    let num_rolls = match args.len() {
        0 => 1,
        1 => {
            let num_rolls = args[0].parse::<InputType>();
            if num_rolls.is_err() {
                let response = format!("Invalid number of rolls: {}", args[0]);
                msg.reply(ctx, response).await?;
                return Ok(());
            }
            num_rolls.unwrap()
        }
        n => {
            let response = format!("Too many arguments (expected 0 or 1, got {})", n);
            msg.reply(ctx, response).await?;
            return Ok(());
        }
    };

    // Make the roll.
    let roll = OracleRoll::random(num_rolls.into());
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
    let spec_raw = {
        let arg = msg.content.split_once(' ');
        if let Some((_, spec_raw)) = arg {
            spec_raw.trim()
        } else {
            let response = "Not enough arguments (expected 1+, got 0)";
            msg.reply(ctx, response).await?;
            return Ok(());
        }
    };
    let spec = if let Ok(spec) = spec_raw.parse() {
        spec
    } else {
        let response = "Invalid roll specification";
        msg.reply(ctx, response).await?;
        return Ok(());
    };

    // Make the roll.
    let roll = CustomRoll::random(spec);
    let response = roll.to_string();

    // Delete the message and respond to it.
    msg.delete(ctx).await?;
    send!(ctx, msg, response).await?;

    Ok(())
}

/// Display a help message.
#[command]
#[aliases("h")]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    const HELP_TEXT: &str = "***Starforged Bot Guide***
*This bot helps you make all the rolls you need. It supports the following commands:*

Action Rolls (`!move`, `!action`, `!ar`, `!a`):
   Roll an action d6 against the challenge 2d10.
   Optionally specify a list of bonuses (i.e. stats and adds); \
this will calculate your total score and tell you the outcome.
   Example: `!action 3 2`

Progress Rolls (`!progress`, `!pr`, `!p`):
   Roll your progress against the challenge 2d10.
   Optionally specify your progress amount (i.e. the number of \
filled boxes); this will tell you the outcome.
   Example: `!p 9`

Oracle Rolls (`!oracle`, `!or`, `!o`):
   Roll a d100 to pick from an oracle table.
   You may specify a number to roll multiple oracles at once.
   Example: `!oracle 3`

Custom rolls (`!roll`, `!r`):
   Roll any dice and bonuses you want, using the format `XdY + Z`.
   You may specify multiple dice and multiple bonuses.
   Example: `!r 2d4 + 1 + d6 + 4d10`

Note that all numbers are limited to 255, i.e. you cannot roll 2d1000 \
or ask for 300 oracle rolls.";

    // Delete the message and respond to it.
    msg.delete(ctx).await?;
    send!(ctx, msg, HELP_TEXT).await?;

    Ok(())
}
