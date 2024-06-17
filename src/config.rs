use config::File;
use serde::Deserialize;

use crate::auth;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub serveraddr: String,
    pub pingtimeout: i64,
    pub pingthreshold: u64,
    pub timeoffset: i8,
    pub auth: Auth,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub method: auth::Methods,
    pub secret: String,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .set_default("serveraddr", "0.0.0.0:7229")?
            .set_default("pingtimeout", 10)?
            .set_default("pingthreshold", 1)?
            .set_default("timeoffset", 0)?
            .set_default("auth.method", "none")?
            .set_default("auth.secret", "")?
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("config.dev.toml").required(false))
            .add_source(config::Environment::with_prefix("WEBOL").separator("_"))
            .build()?;

        config.try_deserialize()
    }
}
