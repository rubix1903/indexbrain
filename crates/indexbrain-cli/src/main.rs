use anyhow::Result;
use tracing_subscriber;
use indexbrain_core::Settings;
use indexbrain_collector::Collector;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load config from config.yaml or env vars
    let settings = Settings::from_file_and_env("config.yaml")?;
    tracing::info!("Loaded configuration: {:?}", settings);

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

        // For now just dump to stdout as JSON
        println!("{}", serde_json::to_string_pretty(&snapshot)?);

        tokio::time::sleep(std::time::Duration::from_secs(settings.database.poll_interval_secs)).await;
    }
}