mod commands;

use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::env;

use commands::{movie, ping, steam};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data;

#[tokio::main]
async fn main() {
    // Get env vars from .env file
    dotenv::dotenv().expect("Failed to load the .env file.");

    // Initialize the logger
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level("learn_bot", LevelFilter::Info)
        .init()
        .unwrap_or_else(|err| {
            eprintln!(
                "Something went wrong while initializing SimpleLogger: {}",
                err
            );
        });

    // Initialize the bot token
    let token = env::var("BOT_TOKEN").expect("Expected a BOT_TOKEN environment variable.");

    if let Err(why) = poise::Framework::build()
        .prefix("&")
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data) }))
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                case_insensitive_commands: true,
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),
                ..Default::default()
            },
            listener: |_ctx, event, _callback, _| {
                Box::pin(async move {
                    if event.name() == "Ready" {
                        info!("Bot is up and running.");
                    }

                    Ok(())
                })
            },
            ..Default::default()
        })
        .command(help(), |f| f)
        .command(register(), |f| f)
        .command(ping(), |f| f)
        .command(movie(), |f| f)
        .command(steam(), |f| f)
        .run()
        .await
    {
        error!("Something went wrong while building the framework: {}", why);
    }
}

/// Show help for a command.
///
/// Usage: \
/// **Prefix:** `&help`
/// **Slash command:** `/help <command>`
/// *Example:* /help ping
#[poise::command(prefix_command, slash_command, track_edits)]
async fn help(
    ctx: Context<'_>,
    #[description = "Show help for a specific command"] command: Option<String>,
) -> Result<(), Error> {
    if let Err(why) = poise::samples::help(
        ctx,
        command.as_deref(),
        "WIP multipurpose bot built in Rustlang. Supports both prefix and slash commands. ex: /ping or &ping",
        poise::samples::HelpResponseMode::Ephemeral,
    )
    .await
    {
        error!("Could not respond to the help command: {}", why);
    }

    Ok(())
}

/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::samples::register_application_commands(ctx, global).await?;

    Ok(())
}
