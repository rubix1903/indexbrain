use anyhow::Result;
use tracing_subscriber;
use indexbrain_core::Settings;
use indexbrain_collector::Collector;
use indexbrain_features::FeaturePipeline;
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

        let context = pipeline.compute_context(&snapshot)?;
        info!("Context vector: {:?}", context);

        //Generating candidate arms from the snapshot
        let arms = generate_candidates(&settings.planner, &snapshot);
        //Selecting action
        let chosen_arm_idx = bandit.select_action(&context, &arms)?;
        let chosen_arm = &arms[chosen_arm_idx];
        info!("Bandit selected arm {}: {}", chosen_arm_idx, chosen_arm.description);

        // for now, simulate a reward (all arms get 0.0)
        let reward = 0.0;
        bandit.update(&context, chosen_arm_idx, reward);

        tokio::time::sleep(std::time::Duration::from_secs(settings.database.poll_interval_secs)).await;
    }
}