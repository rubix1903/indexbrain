use anyhow::Result;
use indexbrain_core::WorkloadSnapshot;
use serde::Deserialize;
use crate::FeatureExtractor;

#[derive(Debug, Clone)]
pub struct TableScanRatio {
    name: String,
}

#[derive(Deserialize)]
struct Params {
    #[serde(default = "default_name")]
    name: String,
}

fn default_name() -> String { "table_scan_ratio".into() }

impl TableScanRatio {
    pub fn new(params: &serde_yaml::Value) -> Result<Self> {
        let p: Params = serde_yaml::from_value(params.clone())?;
        Ok(Self { name: p.name })
    }
}

impl FeatureExtractor for TableScanRatio {
    fn name(&self) -> &str { &self.name }

    fn extract(&self, snapshot: &WorkloadSnapshot) -> Result<Vec<f64>> {
        let total_scans: f64 = snapshot.tables.iter()
            .map(|t| (t.seq_scan + t.idx_scan) as f64)
            .sum();
        if total_scans == 0.0 {
            return Ok(vec![0.0]);
        }
        let seq_scans: f64 = snapshot.tables.iter().map(|t| t.seq_scan as f64).sum();
        Ok(vec![seq_scans / total_scans])
    }
}