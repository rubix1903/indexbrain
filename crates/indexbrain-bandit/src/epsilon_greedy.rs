use anyhow::Result;
use rand::Rng;
use indexbrain_core::bandit::BanditConfig;
use super::{Arm, Bandit};

pub struct EpsilonGreedy {
    epsilon: f64,
    q_values: Vec<f64>,
    counts: Vec<usize>,
}

impl EpsilonGreedy {
    pub fn new(config: &indexbrain_core::bandit::BanditConfig) -> Result<Self> {
        Ok(Self {
            epsilon: config.epsilon,
            q_values: vec![0.0; config.num_arms],
            counts: vec![0; config.num_arms],
        })
    }
}
impl Bandit for EpsilonGreedy {
    fn select_action(&mut self, _context: &[f64], arms: &[Arm]) -> Result<usize> {
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.epsilon || self.counts.iter().all(|&c| c == 0) {
            // Explore: pick a random arm
            let idx = rng.gen_range(0..arms.len());
            Ok(idx)
        } else {
            // Exploit - picking the arm with highest estimated reward
            let best = self.q_values.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);
            Ok(best)
        }
    }

    fn update(&mut self, _context: &[f64], arm_index: usize, reward: f64) {
        if arm_index >= self.q_values.len() {
            return;
        }
        self.counts[arm_index] += 1;
        let n = self.counts[arm_index] as f64;
        self.q_values[arm_index] += (reward - self.q_values[arm_index]) / n;
    }
}