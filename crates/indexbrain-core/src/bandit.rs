use serde::Deserialize;

//Configuration for bandit algorithm
#[derive(Debug, Deserialize, Clone)]
pub struct BanditConfig {
    pub algorithm: String,
    #[serde(default = "default_epsilon")]
    pub epsilon: f64,
    pub num_arms: usize,
    #[serde(default)]
    pub reward: RewardConfig,
}

fn default_epsilon() -> f64 { 0.1 }
#[derive(Debug, Deserialize, Clone)]
pub struct RewardConfig {
    pub improvement_weight: f64,
    pub storage_penalty_factor: f64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self{
            improvement_weight: 0.1,
            storage_penalty_factor: 0.1,
        }
    }
}