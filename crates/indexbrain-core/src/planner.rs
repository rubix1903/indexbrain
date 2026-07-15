use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PlannerConfig {
    pub max_candidates_per_table: usize,
    pub max_total_candidates: usize,
}