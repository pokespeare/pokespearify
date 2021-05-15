pub mod pokeapi;

/// Possible errors from external API calls.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    JsonDecoding(#[from] serde_json::Error),
    #[error(transparent)]
    RateLimit(reqwest::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error("Unspecified API Error")]
    Unspecified,
}
