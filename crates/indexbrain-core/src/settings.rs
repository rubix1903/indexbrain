use serde::Deserialize;
use config::{Config, File, Environment};
use crate::features::FeaturesConfig;
use crate::bandit::BanditConfig;
use crate::planner::PlannerConfig;
use crate::whatif::WhatIfConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub poll_interval_secs: u64,
    pub required_extensions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectorConfig {
    pub max_queries: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub collector: CollectorConfig,
    pub features: FeaturesConfig,
    pub bandit: BanditConfig,
    pub planner: PlannerConfig,
    pub whatif: WhatIfConfig,
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