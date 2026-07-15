use serde::Deserialize;

/// Configuration for a feature extraction pipeline.
#[derive(Debug, Deserialize, Clone)]
pub struct FeaturesConfig {
    pub features: Vec<FeatureDef>,
}

/// Definition of a single feature.
#[derive(Debug, Deserialize, Clone)]
pub struct FeatureDef {
    /// Type of the feature (e.g., "query_mix_ratio", "avg_exec_time").
    pub feature_type: String,
    /// Human‑readable name (optional, defaults to feature_type).
    #[serde(default)]
    pub name: Option<String>,
    /// Normalisation method: "none", "minmax", "zscore".
    #[serde(default)]
    pub normalisation: Normalisation,
    /// Parameters specific to the feature type.
    #[serde(default)]
    pub params: serde_yaml::Value,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum Normalisation {
    #[default]
    None,
    MinMax {
        min: f64,
        max: f64,
    },
    ZScore {
        mean: f64,
        std: f64,
    },
}

impl Normalisation {
    pub fn apply(&self, value: f64) -> f64 {
        match self {
            Normalisation::None => value,
            Normalisation::MinMax { min, max } => {
                (value - min) / (max - min)
            }
            Normalisation::ZScore { mean, std } => {
                (value - mean) / std
            }
        }
    }
}