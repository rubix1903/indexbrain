use anyhow::Result;
use tracing_subscriber;
use indexbrain_core::Settings;
use indexbrain_collector::Collector;
use indexbrain_features::FeaturePipeline;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load config from config.yaml or env vars
    let settings = Settings::from_file_and_env("config.yaml")?;
    tracing::info!("Loaded configuration: {:?}", settings);

    let pipeline = FeaturePipeline::from_config(&settings.features)?;

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

        tokio::time::sleep(std::time::Duration::from_secs(settings.database.poll_interval_secs)).await;
    }
}