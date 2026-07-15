use anyhow::Result;
use indexbrain_core::WorkloadSnapshot;
use serde::Deserialize;
use crate::FeatureExtractor;

#[derive(Debug, Clone)]
pub struct QueryMixRatio {
    name: String,
}

#[derive(Deserialize)]
struct Params {
    #[serde(default = "default_name")]
    name: String,
}

fn default_name() -> String { "query_mix_ratio".into() }

impl QueryMixRatio {
    pub fn new(params: &serde_yaml::Value) -> Result<Self> {
        let p: Params = serde_yaml::from_value(params.clone())?;
        Ok(Self { name: p.name })
    }
}

impl FeatureExtractor for QueryMixRatio {
    fn name(&self) -> &str { &self.name }

    fn extract(&self, snapshot: &WorkloadSnapshot) -> Result<Vec<f64>> {
        let total_calls: f64 = snapshot.queries.iter().map(|q| q.calls as f64).sum();
        if total_calls == 0.0 {
            return Ok(vec![0.0; 4]);
        }
        let mut selects = 0.0;
        let mut inserts = 0.0;
        let mut updates = 0.0;
        let mut deletes = 0.0;
        for q in &snapshot.queries {
            let calls = q.calls as f64;
            let q_upper = q.query_text.trim_start().to_uppercase();
            if q_upper.starts_with("SELECT") {
                selects += calls;
            } else if q_upper.starts_with("INSERT") {
                inserts += calls;
            } else if q_upper.starts_with("UPDATE") {
                updates += calls;
            } else if q_upper.starts_with("DELETE") {
                deletes += calls;
            }
        }
        Ok(vec![
            selects / total_calls,
            inserts / total_calls,
            updates / total_calls,
            deletes / total_calls,
        ])
    }
}