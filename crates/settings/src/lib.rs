use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use crate::human_duration::HumanDuration;

mod human_duration;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub port: u16,
    pub db: Db,
    pub jwt: Auth,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Db {
    url: String,
    pub shema: String,
}

impl Db {
    pub fn get_url(&self) -> String {
        self.url.clone() + "?search_path=" + &self.shema
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Auth {
    pub secret: String,
    pub deadline: HumanDuration,
}

impl Settings {
    pub fn new(
        default_port: Option<String>,
        default_db_shema: Option<String>,
    ) -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        let setting_path = current_dir.join("Settings.yaml");

        let s = Config::builder()
            .set_default("port", default_port.unwrap_or(3000.to_string()))?
            .set_default("db.shema", default_db_shema.unwrap_or("public".to_string()))?
            .set_default("jwt.secret", "Secret")?
            .set_default("jwt.deadline", "24h")?
            .add_source(File::from(setting_path))
            .add_source(Environment::default().separator("_").try_parsing(true))
            .build()?;
        s.try_deserialize()
    }
}
