use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    /// Host or IP that our Server is listening on
    pub host: String,
    /// Port that our Server is listening on
    pub port: u16,
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
