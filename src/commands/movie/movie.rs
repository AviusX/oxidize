use crate::{Context, Error};

use log::error;
use poise::serenity_prelude as serenity;
use serde::Deserialize;
use std::env;

/// Lookup movie details (Plot, IMDb Rating, etc)
///
/// Usage:
/// **Prefix:** `&movie <movie_name>`
/// **Slash command:** `/movie <movie_name>`
#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn movie(
    ctx: Context<'_>,
    #[description = "The name of the movie you want to search"]
    #[rest] // To collect all of the following arguments in prefix command into movie
    movie: String,
) -> Result<(), Error> {
    // Get the API key for the OMDb API from the environment vars
    let api_key = env::var("OMDB_API_KEY").expect("Expected environment variable OMDB_API_KEY");
    let url = format!(
        "http://www.omdbapi.com/?apikey={}&t={}&type=movie",
        api_key, movie
    );

    // Fetch the data for the requested movie from the API
    let api_response = reqwest::get(url)
        .await?
        .json::<Movie>()
        .await
        .unwrap_or(Movie {
            response: "False".to_string(),
            ..Default::default()
        });

    if api_response.response == "False" {
        poise::say_reply(ctx, format!("No movie with title: \"{}\" found.", movie)).await?;
        return Ok(());
    }

    if let Err(why) = poise::send_reply(ctx, |message| {
        message
            .embed(|embed| {
                // Construct an embed
                embed.title(format!("{} ({})", &api_response.title, &api_response.year));
                embed.description(&api_response.plot);
                if &api_response.poster.to_lowercase() != "n/a" {
                    embed.thumbnail(&api_response.poster);
                    embed.image(&api_response.poster);
                }
                embed.author(|author| {
                    if let Some(icon_url) = ctx.author().avatar_url() {
                        author.icon_url(icon_url);
                    } else {
                        author.icon_url(ctx.author().default_avatar_url());
                    }
                    author.name(&ctx.author().name);
                    author
                });

                let imdb_rating = format!("⭐ {}", &api_response.imdb_rating);

                let fields = vec![
                    // (field_name, field_content, inline)
                    ("IMDB Rating: ", &imdb_rating, false),
                    ("Country", &api_response.country, true),
                    ("Rated", &api_response.rated, true),
                    ("Runtime", &api_response.runtime, true),
                    ("Genre", &api_response.genre, false),
                    ("Actors", &api_response.actors, false),
                    ("Released", &api_response.released, false),
                    ("Production", &api_response.production, true),
                    ("Director", &api_response.director, true),
                    ("Writer", &api_response.writer, true),
                    ("Awards", &api_response.awards, false),
                    ("Box Office", &api_response.box_office, true),
                ];

                embed.fields(fields);

                embed.footer(|footer| {
                    if let Some(icon_url) = &ctx.discord().cache.current_user().avatar_url() {
                        footer.icon_url(icon_url);
                    } else {
                        footer.icon_url(ctx.discord().cache.current_user().default_avatar_url());
                    }
                    footer.text(format!(
                        "{} | Movie",
                        ctx.discord().cache.current_user().name
                    ));
                    footer
                });

                embed.timestamp(chrono::Utc::now());
                embed.colour(serenity::Colour::from_rgb(245, 197, 24));

                embed
            })
            .components(|components| {
                // Add a button at the bottom of the embed
                components.create_action_row(|action_row| {
                    action_row.create_button(|button| {
                        button
                            .style(serenity::ButtonStyle::Link)
                            .label("Open IMDb page")
                            .url(format!("https://imdb.com/title/{}", &api_response.imdb_id))
                    })
                })
            })
    })
    .await
    {
        error!("Couldn't respond to the movie command: {}", why);
    };

    Ok(())
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
struct Movie {
    title: String,
    plot: String,
    year: String,
    rated: String,
    released: String,
    runtime: String,
    genre: String,
    director: String,
    writer: String,
    actors: String,
    country: String,
    awards: String,
    #[serde(rename = "imdbRating")]
    imdb_rating: String,
    #[serde(rename = "imdbID")]
    imdb_id: String,
    box_office: String,
    production: String,
    poster: String,
    response: String,
}
