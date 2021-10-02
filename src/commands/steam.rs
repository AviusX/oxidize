use crate::{Context, Error};

use serde::Deserialize;
use poise::serenity_prelude as serenity;
use std::env;

/// Lookup information on games or users on steam
///
/// **Usage:**
///
/// **User command**
/// `&steam "Get user information" <vanity_name>`
///     ex: `&steam "Get user information" robinwalker`
#[poise::command(prefix_command, slash_command, track_edits)]
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
            let url = format!(
                "https://api.steampowered.com/ISteamUser/ResolveVanityURL/v0001/?key={}&vanityurl={}", 
                api_key, 
                arg
            );

            // get the steam_id of the user that we can use with other commands.
            let api_response = reqwest::get(url).await?.json::<SteamIDResponse>().await?.response;

            if let Some(steam_id) = &api_response.steam_id {
                // if the user has been found, get their info
                let url = format!(
                    "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
                    api_key,
                    steam_id
                );

                let api_response = &reqwest::get(url)
                    .await?
                    .json::<PlayerSummaryResponse>()
                    .await?
                    .response
                    .players[0];
                
                // send the embed
                poise::send_reply(ctx, |message| {
                    message.embed(|embed| {
                        embed.title(&api_response.persona_name).url(&api_response.profile_url);
                        embed.thumbnail(&api_response.avatar);
                        embed.colour(serenity::Colour::from_rgb(0, 0, 0));

                        embed.field("SteamID", &api_response.steam_id, false);

                        if let Some(time_created) = api_response.time_created {
                            let created_at = chrono
                                ::NaiveDateTime
                                ::from_timestamp(time_created, 0)
                                .format("%B %e, %Y");
                        
                            embed.field("Account created", created_at, true);
                        }

                        if let Some(last_logoff) = api_response.last_logoff {
                            let last_logoff = chrono
                                ::NaiveDateTime
                                ::from_timestamp(last_logoff, 0)
                                .format("%B %e, %Y");
                        
                            embed.field("Last logoff", last_logoff, true);
                        }

                        if let Some(game_extra_info) = &api_response.game_extra_info {
                            embed.field("Currently playing", game_extra_info, false);
                        }

                        if api_response.community_visibility_state == 1 {
                            embed.field("Account privacy", "Private", false);
                        } else {
                            embed.field("Account privacy", "Public", false);
                        }
                        
                        embed
                    })
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                action_row.create_button(|button| {
                                    button
                                        .style(serenity::ButtonStyle::Link)
                                        .label("Open user profile")
                                        .url(&api_response.profile_url)
                                })
                            })
                        })
                })
                .await?;
            } else {
                // else, send an error message
                poise::say_reply(ctx, format!("User with vanity name \"{}\" not found.", arg)).await?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, poise::SlashChoiceParameter)]
pub enum Subcommand {
    #[name = "Get user information"]
    User,
}

#[derive(Debug, Deserialize)]
struct SteamIDResponse {
    response: SteamID
}

#[derive(Debug, Deserialize)]
struct SteamID {
    #[serde(rename = "steamid")]
    steam_id: Option<String>,
    success: i32,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlayerSummaryResponse {
    response: Players
}

#[derive(Debug, Deserialize)]
struct Players {
    players: Vec<PlayerSummary>
}

#[derive(Debug, Deserialize)]
struct PlayerSummary {
    #[serde(rename = "steamid")]
    steam_id: String,

    #[serde(rename = "personaname")]
    persona_name: String,

    #[serde(rename = "avatarmedium")]
    avatar: String,

    #[serde(rename = "lastlogoff")]
    last_logoff: Option<i64>,

    #[serde(rename = "timecreated")]
    time_created: Option<i64>,

    #[serde(rename = "communityvisibilitystate")]
    community_visibility_state: u8,

    #[serde(rename = "profileurl")]
    profile_url: String,

    #[serde(rename = "gameextrainfo")]
    game_extra_info: Option<String> 
}