use crate::{Context, Error};

use serde::Deserialize;
use poise::serenity_prelude as serenity;
use rand::seq::SliceRandom;

const CUSTOM_SEARCH_URL: &str = "https://customsearch.googleapis.com/customsearch/v1";
const GOOGLE_LOGO: &str = "https://www.freepnglogos.com/uploads/google-logo-png/google-logo-icon-png-transparent-background-osteopathy-16.png";
const GOOGLE_COLORS: [serenity::Colour; 4] = [
    serenity::Colour::from_rgb(66, 133, 244), // google blue
    serenity::Colour::from_rgb(219, 68, 55), // google red
    serenity::Colour::from_rgb(244, 180, 0), // google yellow
    serenity::Colour::from_rgb(15, 157, 88), // google green
];

/// Search Google from Discord and get the top results displayed in an embed
///
/// **Usage:**
/// `/google <query>`
///
/// **Example:**
/// `/google shiba inu`
#[poise::command(
prefix_command,
slash_command,
broadcast_typing,
defer_response,
track_edits,
aliases("search")
)]
pub async fn google(
    ctx: Context<'_>,
    #[description = "Your Google search query"]
    #[rest] query: String,
) -> Result<(), Error> {
    let api_key = dotenv::var("GOOGLE_SEARCH_KEY")
        .expect("Expected environment variable GOOGLE_SEARCH_KEY");
    let search_engine_id = dotenv::var("SEARCH_ENGINE_ID")
        .expect("Expected environment variable SEARCH_ENGINE_ID");

    let search_url = format!(
        "{}?key={}&cx={}&num={}&q={}",
        CUSTOM_SEARCH_URL,
        api_key,
        search_engine_id,
        5, // the number of results to get
        query // the query to search for
    );

    let search_result = reqwest::get(&search_url)
        .await?
        .json::<Response>()
        .await?;

    poise::send_reply(ctx, |message| {
        message.embed(|embed| {
            embed.title("Google Search Results");

            // randomly choose one of the 4 Google colors and assign that to the embed
            let mut rng = rand::thread_rng();
            let embed_color = *GOOGLE_COLORS.choose(&mut rng).unwrap();
            embed.colour(embed_color);

            embed.thumbnail(GOOGLE_LOGO);
            embed.author(|author| {
                if let Some(icon_url) = ctx.author().avatar_url() {
                    author.icon_url(icon_url);
                } else {
                    author.icon_url(ctx.author().default_avatar_url());
                }
                author.name(&ctx.author().name);
                author
            });

            for search_result in &search_result.items {
                embed.field(
                    &search_result.title,
                    format!("**[Link]({})**\n{}", &search_result.link, &search_result.snippet),
                    false,
                );
            }

            embed.footer(|footer| {
                if let Some(icon_url) = &ctx.discord().cache.current_user().avatar_url() {
                    footer.icon_url(icon_url);
                } else {
                    footer
                        .icon_url(ctx.discord().cache.current_user().default_avatar_url());
                }
                footer.text(format!(
                    "{} | Google",
                    ctx.discord().cache.current_user().name
                ));
                footer
            });

            embed.timestamp(chrono::Utc::now());

            embed
        });

        message
    })
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Response {
    items: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    title: String,
    link: String,
    snippet: String,
}
