use anyhow::Result;
use indexbrain_core::WorkloadSnapshot;
use serde::Deserialize;
use crate::FeatureExtractor;

#[derive(Debug, Clone)]
pub struct AvgExecTime {
    name: String,
}

#[derive(Deserialize)]
struct Params {
    #[serde(default = "default_name")]
    name: String,
}

fn default_name() -> String { "avg_exec_time".into() }

impl AvgExecTime {
    pub fn new(params: &serde_yaml::Value) -> Result<Self> {
        let p: Params = serde_yaml::from_value(params.clone())?;
        Ok(Self { name: p.name })
    }
}

impl FeatureExtractor for AvgExecTime {
    fn name(&self) -> &str { &self.name }

    fn extract(&self, snapshot: &WorkloadSnapshot) -> Result<Vec<f64>> {
        let total_calls: f64 = snapshot.queries.iter().map(|q| q.calls as f64).sum();
        if total_calls == 0.0 {
            return Ok(vec![0.0]);
        }
        let weighted_mean = snapshot.queries.iter()
            .map(|q| q.mean_exec_time * q.calls as f64)
            .sum::<f64>() / total_calls;
        Ok(vec![weighted_mean])
    }
}