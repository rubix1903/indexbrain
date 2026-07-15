use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct WhatIfConfig {
    pub storage_penalty_factor: f64,
    pub improvement_weight: f64,
}