use config::Config;
use once_cell::sync::Lazy;

pub static SETTINGS: Lazy<Config> = Lazy::new(setup);

fn setup() -> Config {
    Config::builder()
        .add_source(config::Environment::with_prefix("WEBOL").separator("_"))
        .build()
        .unwrap()
}