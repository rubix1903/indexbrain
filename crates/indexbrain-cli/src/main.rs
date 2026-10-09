use anyhow::Result;
use tracing_subscriber;
use indexbrain_core::Settings;
use indexbrain_collector::Collector;
use indexbrain_features::FeaturePipeline;
use indexbrain_whatif::evaluate_arm;
use tracing::info;
use indexbrain_bandit::create_bandit;
use indexbrain_planner::generate_candidates;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load config from config.yaml or env vars
    let settings = Settings::from_file_and_env("config.yaml")?;
    tracing::info!("Loaded configuration: {:?}", settings);

    let pipeline = FeaturePipeline::from_config(&settings.features)?;
    let mut bandit = create_bandit(&settings.bandit)?;

    // Build collector
    let collector = Collector::new(
        &settings.database,
        settings.collector.max_queries,
    ).await?;

    // Main loop: poll periodically
    loop {
        let snapshot = collector.take_snapshot().await?;
        tracing::info!("Snapshot taken: {} queries, {} tables",
        snapshot.queries.len(), snapshot.tables.len());

        // Compute context vector
        let context = pipeline.compute_context(&snapshot)?;
        info!("Context vector: {:?}", context);

        // Generate candidate arms
        let arms = generate_candidates(&settings.planner, &snapshot);

        // Bandit selects an arm
        let chosen_arm_idx = bandit.select_action(&context, &arms)?;
        let chosen_arm = &arms[chosen_arm_idx];
        info!("Bandit selected arm {}: {}", chosen_arm_idx, chosen_arm.description);

        // Prepare query texts from SELECT statements for what-if evaluation
        let mut query_texts: Vec<String> = snapshot.queries.iter()
            .filter(|q| q.query_text.trim_start().to_uppercase().starts_with("SELECT"))
            .take(5)
            .map(|q| q.query_text.clone())
            .collect();
        if query_texts.is_empty() {
            query_texts.push("SELECT 1".into());
        }

        // Call the real what-if engine instead of dummy zero reward
        let reward = evaluate_arm(collector.pool(), &settings.whatif, chosen_arm, &query_texts).await
            .unwrap_or_else(|e| {
                tracing::error!("What‑if evaluation failed: {}", e);
                0.0
            });
        info!("Reward: {}", reward);

        // Update bandit with the actual reward
        bandit.update(&context, chosen_arm_idx, reward);

        tokio::time::sleep(std::time::Duration::from_secs(settings.database.poll_interval_secs)).await;
    }
}