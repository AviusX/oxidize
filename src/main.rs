mod commands;
mod helpers;

use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::env;
use tokio::sync::RwLock;

use commands::{code, movie, ping, steam, user, clear, google};
use helpers::{get_versions, Runtimes};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    runtimes: RwLock<Runtimes>,
}

#[tokio::main]
async fn main() {
    // Get env vars from .env file
    dotenv::dotenv().expect("Failed to load the .env file.");

    // Initialize the logger
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level("oxidize", LevelFilter::Debug)
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
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {
            Ok(
                Data {
                    runtimes: RwLock::new(Vec::new())
                }
            )
        }))
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                case_insensitive_commands: true,
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),
                ..Default::default()
            },
            listener: |_ctx, event, _callback, data| {
                Box::pin(async move {
                    if event.name() == "Ready" {
                        info!("Starting the bot...");
                        info!("Getting runtime data from Piston...");

                        // Try fetching data from piston 3 times and fail bot startup if unsuccessful
                        let mut retries = 1;
                        while retries <= 3 {
                            if let Ok(response) = get_versions().await {
                                let mut runtimes = data.runtimes.write().await;
                                for runtime in response {
                                    runtimes.push(runtime);
                                }

                                info!("Successfully fetched the runtimes from the Piston API.");
                                break;
                            } else {
                                error!("Error getting runtime data. Trying again [{}/3]", retries);
                                retries += 1;
                            }
                        }

                        // If language runtimes weren't fetched from Piston after 3 retries, panic.
                        if retries == 4 {
                            error!("Failed to fetch runtimes from the Piston API after 3 retries. Exiting.");
                            std::process::exit(1);
                        }

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
        .command(steam(), |f| f.subcommand(user(), |s| s))
        .command(code(), |f| f)
        .command(clear(), |f| f)
        .command(google(), |f| f)
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
        "WIP multipurpose bot built in Rustlang. Supports both prefix and slash commands. ex: /ping or &ping. The code execution command currently only works with prefix commands (&code)",
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

#[derive(Debug, poise::SlashChoiceParameter)]
enum Commands {
    #[name = "help"]
    Help,
    #[name = "ping"]
    Ping,
    #[name = "movie"]
    Movie,
    #[name = "steam"]
    Steam,
}
