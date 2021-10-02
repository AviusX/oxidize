use crate::{Context, Error};
use chrono::Utc;
use log::error;

/// Ping command to test bot response time.
///
/// Usage:
/// **Prefix:** `&ping`
/// **Slash command:** `/ping`
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response_time =
        Utc::now().timestamp_subsec_millis() - ctx.created_at().timestamp_subsec_millis();
    let response = format!("Pong! Response time: `{}ms`", response_time);

    if let Err(why) = poise::say_reply(ctx, response).await {
        error!("Couldn't send reply for ping: {}", why);
    }

    Ok(())
}
