use crate::{Context, Error};
use tokio::time::{Duration, sleep};

/// Delete messages in bulk.
///
/// Maximum messages that can be deleted at once are 100.
/// **Usage:**
/// `/clear <number>`
///
/// **Example:**
/// `/clear 50` will delete the last 50 messages.
#[poise::command(
    prefix_command,
    slash_command,
    broadcast_typing,
    defer_response,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn clear(
    ctx: Context<'_>,
    #[description = "The number of messages to delete"] number: u64,
) -> Result<(), Error> {
    // get the last `number` messages
    let messages = ctx
        .channel_id()
        .messages(&ctx.discord().http, |messages| {
            messages.before(ctx.id());
            messages.limit(number);

            messages
        })
        .await?;

    // delete the acquired messages
    if let Err(err) = ctx
        .channel_id()
        .delete_messages(
            &ctx.discord().http,
            messages.iter().map(|message| message.id),
        )
        .await
    {
        poise::say_reply(ctx, format!("There was an error: {}", err)).await?;
    } else {
        let reply = poise::say_reply(
            ctx,
            format!("Successfully deleted the last {} messages.", number),
        )
        .await?;

        // Wait 3 seconds and delete the notification message
        sleep(Duration::from_secs(3)).await;
        reply.message().await?.delete(&ctx.discord().http).await?;
    }

    Ok(())
}
