use reqwest::Error;
use serde::Deserialize;

const PISTON_RUNTIME_URL: &str = "https://emkc.org/api/v2/piston/runtimes";

/// Makes a call to the Piston API to get the available runtimes
/// and returns the response.
pub async fn get_versions() -> Result<Runtimes, Error> {
    let response = reqwest::get(PISTON_RUNTIME_URL)
        .await?
        .json::<Runtimes>()
        .await?;

    Ok(response)
}

pub type Runtimes = Vec<Language>;

#[derive(Debug, Deserialize)]
pub struct Language {
    pub language: String,
    pub version: String,
    pub aliases: Vec<String>,
}
