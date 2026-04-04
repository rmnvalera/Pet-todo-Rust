use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use crate::utils::human_duration::HumanDuration;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub port: u16,
    pub db_url: String,
    pub auth: Auth,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Auth {
    pub jwt_secret: String,
    pub jwt_deadline: HumanDuration,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        let setting_path = current_dir.join("Settings.yaml");

        let s = Config::builder()
            .add_source(File::from(setting_path))
            .add_source(
                Environment::with_prefix("")
                    .prefix_separator("")
                    .separator("_"),
            )
            .build()?;
        s.try_deserialize()
    }
}
