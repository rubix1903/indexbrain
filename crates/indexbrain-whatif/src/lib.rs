use anyhow::{Result, Context};
use indexbrain_core::Arm;
use sqlx::PgPool;
use tracing::info;

pub async fn evaluate_arm(
    pool: &PgPool,
    config: &indexbrain_core::whatif::WhatIfConfig,
    arm: &Arm,
    query_texts: &[String],
) -> Result<f64> {
    if query_texts.is_empty() {
        return Ok(0.0);
    }
    let query = &query_texts[0];
    let col_list = arm.columns.join(", ");
    let create_stmt = format!("CREATE INDEX ON {}.{} ({})", arm.schema, arm.table_name, col_list);

    let mut tx = pool.begin().await?;

    // Baseline cost
    let baseline_cost = get_query_cost(&mut *tx, query).await?;

    // Create hypothetical index
    sqlx::query(&format!("SELECT hypopg_create_index('{}')", create_stmt))
        .execute(&mut *tx)
        .await
        .context("Failed to create hypothetical index")?;

    // Cost with hypothetical index
    let hypo_cost = get_query_cost(&mut *tx, query).await?;

    // Reset hypopg
    sqlx::query("SELECT hypopg_reset()")
        .execute(&mut *tx)
        .await?;

    // Rollback (no permanent changes)
    tx.rollback().await?;

    if baseline_cost == 0.0 {
        return Ok(0.0);
    }
    let improvement = (baseline_cost - hypo_cost) / baseline_cost;
    let reward = config.improvement_weight * improvement
        - config.storage_penalty_factor * (arm.columns.len() as f64);
    info!(
        "Arm {}: baseline={}, hypo={}, reward={}",
        arm.description, baseline_cost, hypo_cost, reward
    );
    Ok(reward)
}

async fn get_query_cost(
    tx: &mut sqlx::PgConnection,
    query: &str,
) -> Result<f64> {
    let explain_stmt = format!("EXPLAIN (FORMAT JSON) {}", query);
    let row: (serde_json::Value,) = sqlx::query_as(&explain_stmt)
        .fetch_one(tx)
        .await?;
    let plan = &row.0[0]["Plan"];
    let total_cost = plan["Total Cost"].as_f64().unwrap_or(0.0);
    Ok(total_cost)
}