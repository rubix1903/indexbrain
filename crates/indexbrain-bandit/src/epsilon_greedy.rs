use anyhow::Result;
use rand::Rng;
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
        let n_arms = arms.len();
        if n_arms == 0 {
            anyhow::bail!("No arms available");
        }
        if self.q_values.len() < n_arms {
            self.q_values.resize(n_arms, 0.0);
            self.counts.resize(n_arms, 0);
        }
        if rng.gen::<f64>() < self.epsilon || self.counts.iter().take(n_arms).all(|&c| c == 0) {
            // Explore: pick a random valid arm
            let idx = rng.gen_range(0..n_arms);
            Ok(idx)
        } else {
            let best = self.q_values.iter()
                .take(n_arms)
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