use anyhow::Result;
use tracing_subscriber;
use indexbrain_core::Settings;
use indexbrain_collector::Collector;
use indexbrain_features::FeaturePipeline;
use tracing::info;
use indexbrain_bandit::create_bandit;
use indexbrain_bandit::Arm;
use indexbrain_core::WorkloadSnapshot;

fn generate_arms(snapshot: &WorkloadSnapshot, max_arms: usize) -> Vec<Arm> {
    let mut arms = Vec::new();
    for table in &snapshot.tables {
        if arms.len() >= max_arms {
            break;
        }
        // For each table,proposing a dummy single‑column index on the first column we can guess?
        // We don't have column info yet, so we use a placeholder column name "???".
        arms.push(Arm {
            id: arms.len(),
            description: format!("CREATE INDEX ON {}.{}(???);", table.schema, table.table_name),
        });
    }
    // If no tables, add a fallback arm
    if arms.is_empty() {
        arms.push(Arm {
            id: 0,
            description: "No candidate indexes".into(),
        });
    }
    arms
}
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
        let arms = generate_arms(&snapshot, settings.bandit.num_arms);
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