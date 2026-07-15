use anyhow::Result;
use indexbrain_core::WorkloadSnapshot;
use indexbrain_core::features::{FeaturesConfig, Normalisation};

mod features;
use features::query_mix_ratio::QueryMixRatio;
use features::avg_exec_time::AvgExecTime;
use features::table_scan_ratio::TableScanRatio;

/// A single feature that can be extracted from a workload snapshot.
pub trait FeatureExtractor: Send + Sync {
    fn name(&self) -> &str;
    fn extract(&self, snapshot: &WorkloadSnapshot) -> Result<Vec<f64>>;
}

pub struct FeaturePipeline {
    extractors: Vec<Box<dyn FeatureExtractor>>,
    normalisers: Vec<Option<Normalisation>>,
}

impl FeaturePipeline {
    pub fn from_config(config: &FeaturesConfig) -> Result<Self> {
        let mut extractors: Vec<Box<dyn FeatureExtractor>> = Vec::new();
        let mut normalisers = Vec::new();

        for def in &config.features {
            let extractor: Box<dyn FeatureExtractor> = match def.feature_type.as_str() {
                "query_mix_ratio" => Box::new(QueryMixRatio::new(&def.params)?),
                "avg_exec_time"   => Box::new(AvgExecTime::new(&def.params)?),
                "table_scan_ratio" => Box::new(TableScanRatio::new(&def.params)?),
                other => anyhow::bail!("Unknown feature type: {}", other),
            };
            normalisers.push(Some(def.normalisation.clone()));
            extractors.push(extractor);
        }

        Ok(Self { extractors, normalisers })
    }

    pub fn compute_context(&self, snapshot: &WorkloadSnapshot) -> Result<Vec<f64>> {
        let mut ctx = Vec::new();
        for (i, extractor) in self.extractors.iter().enumerate() {
            let mut values = extractor.extract(snapshot)?;
            if let Some(norm) = &self.normalisers[i] {
                for v in &mut values {
                    *v = norm.apply(*v);
                }
            }
            ctx.extend(values);
        }
        Ok(ctx)
    }
}