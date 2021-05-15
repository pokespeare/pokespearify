use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    /// Host or IP that our Server is listening on
    pub host: String,
    /// Port that our Server is listening on
    pub port: u16,
    /// The base URL of the PokéApi
    pub poke_api_base_url: PokeApiUrl,
    /// The base URL of the Translator API
    pub translator_api_base_url: TranslateApiUrl,
}

impl Config {
    /// Collect the Config from Env and config file in the current directory.
    pub fn collect() -> anyhow::Result<Self> {
        // This implementation is based on https://github.com/LukeMathWalker/zero-to-production/blob/main/src/configuration.rs
        let mut settings = config::Config::default();
        let cwd = std::env::current_dir()?;

        settings.merge(config::File::from(cwd.join("config")).required(true))?;
        settings.merge(config::Environment::with_prefix("app").separator("__"))?;

        Ok(settings.try_into()?)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct PokeApiUrl(pub Url);

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct TranslateApiUrl(pub Url);
