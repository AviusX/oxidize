// add commands here
mod code;
mod movie;
mod ping;
mod steam;
mod clear;
mod google;

// re-export the main command functions
pub use code::code::code;
pub use movie::movie::movie;
pub use ping::ping::ping;
pub use steam::{steam::steam, user::user}; // steam main command and the user subcommand
pub use clear::clear::clear;
pub use google::google::google;
