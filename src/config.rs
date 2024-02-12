use config::File;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub apikey: String,
    pub serveraddr: String,
    pub pingtimeout: i64,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .set_default("serveraddr", "0.0.0.0:7229")?
            .set_default("pingtimeout", 10)?
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("config.dev.toml").required(false))
            .add_source(config::Environment::with_prefix("WEBOL").prefix_separator("_"))
            .build()?;

        config.try_deserialize()
    }
}

