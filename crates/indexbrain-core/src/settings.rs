use serde::Deserialize;
use config::{Config, File, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub collector: CollectorConfig,
    // learner, executor, etc. later
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,                // e.g., postgres://...
    pub poll_interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectorConfig {
    pub fetch_query_text: bool,     // whether to retrieve full query text
    pub max_queries: usize,         // limit on pg_stat_statements rows
}

impl Settings {
    pub fn from_file_and_env(config_file: &str) -> Result<Self, config::ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(config_file).required(true))
            .add_source(Environment::with_prefix("INDEXBRAIN").separator("__"))
            .build()?;
        s.try_deserialize()
    }
}