use serde::Deserialize;
use config::{Config, ConfigError, Environment};

#[derive(Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub token: String,
    pub repos: Vec<String>,
}

impl AppConfig {
    pub fn new(config_file: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name(config_file))
            .add_source(Environment::with_prefix("RTRACK"))
            .build()?;
        let res = s.try_deserialize();

        res
    }
}
