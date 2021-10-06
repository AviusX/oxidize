// add commands here
mod code;
mod movie;
mod ping;
mod steam;

// re-export the main command functions
pub use code::code::code;
pub use movie::movie::movie;
pub use ping::ping::ping;
pub use steam::{steam::steam, user::user}; // steam main command
