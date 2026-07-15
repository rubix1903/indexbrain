use anyhow::Result;
use indexbrain_core::bandit::BanditConfig;
use indexbrain_core::Arm;
pub mod epsilon_greedy;
pub use epsilon_greedy::EpsilonGreedy;

// generic bandit interface.
pub trait Bandit: Send + Sync {
    fn select_action(&mut self, context: &[f64], arms: &[Arm]) -> Result<usize>;
    fn update(&mut self, context: &[f64], arm_index: usize, reward: f64);
}

//kinda factory that builds a Bandit from configuration.
pub fn create_bandit(config: &BanditConfig) -> Result<Box<dyn Bandit>> {
    match config.algorithm.as_str() {
        "epsilon_greedy" => {
            let bandit = EpsilonGreedy::new(config)?;
            Ok(Box::new(bandit))
        }
        other => anyhow::bail!("Unknown bandit algorithm: {}", other),
    }
}