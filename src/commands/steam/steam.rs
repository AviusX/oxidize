use crate::{Context, Error};

/// Lookup information on games or users on steam
///
/// **Subcommands**
///
/// **user**
/// `/steam user vanity <vanity_name>`
/// `/steam user id <steam_id>`
/// *examples*
/// `/steam user vanity robinwalker`
/// `/steam user id 76561197972495328`
#[poise::command(prefix_command, slash_command)]
pub async fn steam(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
