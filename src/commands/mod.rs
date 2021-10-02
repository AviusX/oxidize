// add commands here
mod ping;
mod movie;
mod steam;

// re-export the main command functions
pub use steam::steam::steam;
pub use ping::ping::ping;
pub use movie::movie::movie;