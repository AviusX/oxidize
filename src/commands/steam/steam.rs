use super::user::user;
use crate::{Context, Error};

use std::env;

/// Lookup information on games or users on steam
///
/// **Usage:**
///
/// **User command**
/// `&steam "Get user information" <vanity_name>`
///     ex: `&steam "Get user information" robinwalker`
#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    defer_response,
    broadcast_typing
)]
pub async fn steam(
    ctx: Context<'_>,
    #[description = "Steam subcommand to execute"] subcommand: Subcommand,
    #[description = "Argument for the subcommand"] arg: String,
) -> Result<(), Error> {
    // get steam api key from the environment vars
    let api_key = env::var("STEAM_API_KEY").expect("Expected STEAM_API_KEY environment variable.");

    // implement logic for all possible subcommands
    match subcommand {
        Subcommand::User => {
            user(api_key, arg, ctx).await?;
        }
    }

    Ok(())
}

#[derive(Debug, poise::SlashChoiceParameter)]
pub enum Subcommand {
    #[name = "Get user information"]
    User,
}
