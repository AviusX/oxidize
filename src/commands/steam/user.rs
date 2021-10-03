use crate::{Context, Error};
use std::env;

use poise::serenity_prelude as serenity;
use serde::Deserialize;

/// Look up information about a steam user
#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    // defer_response, // currently causes buttons to stop showing up with slash commands
    broadcast_typing
)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "Type of the argument"] arg_type: ArgType,
    #[description = "The vanity name of the user"] arg: String,
) -> Result<(), Error> {
    // get steam api key from the environment vars
    let api_key = env::var("STEAM_API_KEY").expect("Expected STEAM_API_KEY environment variable.");

    let url = format!(
        "https://api.steampowered.com/ISteamUser/ResolveVanityURL/v0001/?key={}&vanityurl={}",
        api_key, arg
    );

    // get the steam_id. either directly from the user, or from the api using the vanity name
    let steam_id = match arg_type {
        ArgType::Id => Some(arg.clone()),
        ArgType::Vanity => {
            // get the steam_id using the vanity name
            reqwest::get(url)
                .await?
                .json::<SteamIDResponse>()
                .await?
                .response
                .steam_id
        }
    };

    if let Some(steam_id) = steam_id {
        // if the user has been found, getappids_filter  their info
        let player_summary_url = format!(
            "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
            api_key, steam_id
        );
        let owned_games_url = format!(
            "http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json",
            api_key, steam_id
        );
        let steam_level_url = format!(
            "http://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key={}&steamid={}&format=json",
            api_key, steam_id
        );
        let vac_bans_url = format!(
            "http://api.steampowered.com/ISteamUser/GetPlayerBans/v1?key={}&steamids={}",
            api_key, steam_id
        );

        // get information about the supplied user
        let player_summary = &reqwest::get(player_summary_url)
            .await?
            .json::<PlayerSummaryResponse>()
            .await?
            .response
            .players[0];

        // get the number of games that the user owns
        let game_count = reqwest::get(owned_games_url)
            .await?
            .json::<OwnedGamesResponse>()
            .await?
            .response
            .game_count;

        // get the account level of the user
        let steam_level = reqwest::get(steam_level_url)
            .await?
            .json::<SteamLevelResponse>()
            .await?
            .response
            .player_level;

        // get information on VAC bans on the user
        let vac_bans = &reqwest::get(vac_bans_url)
            .await?
            .json::<VACBansResponse>()
            .await?
            .players[0];

        // send the embed
        poise::send_reply(ctx, |message| {
            message
                .embed(|embed| {
                    embed
                        .title(format!("{} [{}]", player_summary.persona_name, steam_level))
                        .url(&player_summary.profile_url);
                    embed.thumbnail(&player_summary.avatar);
                    embed.colour(serenity::Colour::from_rgb(0, 0, 0));
                    embed.author(|author| {
                        if let Some(icon_url) = ctx.author().avatar_url() {
                            author.icon_url(icon_url);
                        } else {
                            author.icon_url(ctx.author().default_avatar_url());
                        }
                        author.name(&ctx.author().name);
                        author
                    });

                    embed.field("SteamID", &player_summary.steam_id, false);

                    if let Some(time_created) = player_summary.time_created {
                        let created_at = chrono::NaiveDateTime::from_timestamp(time_created, 0)
                            .format("%B %e, %Y");

                        embed.field("Account created", created_at, true);
                    }

                    // if last_logoff exists, add that to embed
                    if let Some(last_logoff) = player_summary.last_logoff {
                        let last_logoff = chrono::NaiveDateTime::from_timestamp(last_logoff, 0)
                            .format("%B %e, %Y");

                        embed.field("Last logoff", last_logoff, true);
                    }

                    if player_summary.community_visibility_state == 1 {
                        embed.field("Account privacy", "Private", false);
                    } else {
                        embed.field("Account privacy", "Public", false);
                    }

                    // If user has any VAC bans, show that
                    if vac_bans.vac_banned {
                        embed.field(
                            "🚫 VAC bans",
                            format!(
                                "**{}** ({} days ago)",
                                vac_bans.number_of_bans, vac_bans.days_since_last_ban
                            ),
                            false,
                        );
                    }

                    embed.field("Status", get_status(&player_summary.persona_state), false);

                    // if game_extra_info exists, add that to embed
                    if let Some(game_extra_info) = &player_summary.game_extra_info {
                        embed.field("Currently playing", game_extra_info, false);
                    }

                    if let Some(game_count) = game_count {
                        embed.field("Owned games", game_count, false);
                    }

                    embed.footer(|footer| {
                        if let Some(icon_url) = &ctx.discord().cache.current_user().avatar_url() {
                            footer.icon_url(icon_url);
                        } else {
                            footer
                                .icon_url(ctx.discord().cache.current_user().default_avatar_url());
                        }
                        footer.text(format!(
                            "{} | Steam",
                            ctx.discord().cache.current_user().name
                        ));
                        footer
                    });

                    embed.timestamp(chrono::Utc::now());

                    embed
                })
                .components(|components| {
                    components.create_action_row(|action_row| {
                        action_row.create_button(|button| {
                            button
                                .style(serenity::ButtonStyle::Link)
                                .label("Open user profile")
                                .url(&player_summary.profile_url)
                        })
                    })
                })
        })
        .await?;
    } else {
        // else, send an error message
        if let ArgType::Id = arg_type {
            poise::say_reply(ctx, format!("User with steam id \"{}\" not found.", arg)).await?;
        } else {
            poise::say_reply(ctx, format!("User with vanity name \"{}\" not found.", arg)).await?;
        }
    }

    Ok(())
}

#[derive(Debug, poise::SlashChoiceParameter)]
pub enum ArgType {
    #[name = "vanity"]
    Vanity,
    #[name = "id"]
    Id,
}

/// Takes a persona_state u8 and returns the corresponding
/// user status (online, offline, busy, etc)
fn get_status(persona_state: &u8) -> String {
    match persona_state {
        0 => String::from("⚫ Offline"),
        1 => String::from("🟢 Online"),
        2 => String::from("🔴 Busy"),
        3 => String::from("🟡 Away"),
        4 => String::from("🔵 Snooze"),
        5 => String::from("🟠 Looking to trade"),
        6 => String::from("🟣 Looking to play"),
        _ => String::from("⚪ Unknown"),
    }
}

#[derive(Debug, Deserialize)]
struct SteamIDResponse {
    response: SteamID,
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
    response: Players,
}

#[derive(Debug, Deserialize)]
struct Players {
    players: Vec<PlayerSummary>,
}

#[derive(Debug, Deserialize)]
struct PlayerSummary {
    #[serde(rename = "steamid")]
    steam_id: String, // the unique steam_id of the user [public]

    #[serde(rename = "personaname")]
    persona_name: String, // the displayed name or username (not unique on steam) [public]

    #[serde(rename = "avatarfull")]
    avatar: String, // the url for the steam avatar of the user [public]

    #[serde(rename = "lastlogoff")]
    last_logoff: Option<i64>, // the last date of logoff of the user [public]

    #[serde(rename = "timecreated")]
    time_created: Option<i64>, // the time at which the user's account was created [private]

    #[serde(rename = "communityvisibilitystate")]
    community_visibility_state: u8, // 1 if the user's profile is private, 3 if it's public [public]

    #[serde(rename = "profileurl")]
    profile_url: String, // the complete steam url of the user's profile [public]

    #[serde(rename = "gameextrainfo")]
    game_extra_info: Option<String>, // the name of the game that the user is currently playing [private]

    #[serde(rename = "personastate")]
    persona_state: u8,
}

#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: GameCount,
}

#[derive(Debug, Deserialize)]
struct GameCount {
    game_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SteamLevelResponse {
    response: SteamLevel,
}

#[derive(Debug, Deserialize)]
struct SteamLevel {
    player_level: u8,
}

#[derive(Debug, Deserialize)]
struct VACBansResponse {
    players: Vec<VacBans>,
}

#[derive(Debug, Deserialize)]
struct VacBans {
    #[serde(rename = "VACBanned")]
    vac_banned: bool,
    #[serde(rename = "NumberOfVACBans")]
    number_of_bans: u8,
    #[serde(rename = "DaysSinceLastBan")]
    days_since_last_ban: u16,
}
