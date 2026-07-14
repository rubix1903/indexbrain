use anyhow::Result;
use indexbrain_core::{
    settings::DatabaseConfig,
    WorkloadSnapshot, QueryStats, TableStats, IndexInfo,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use chrono::Utc;
use tracing::info;

/// The collector connects to a PostgreSQL database and periodically
/// gathers workload statistics from pg_stat_statements and related views.
pub struct Collector {
    pool: sqlx::PgPool,
    max_queries: usize,
}

/// Ensure all required PostgreSQL extensions are installed.
/// If any are missing, an error is returned with the exact SQL to fix it.
pub async fn validate_extensions(pool: &sqlx::PgPool, required: &[String]) -> Result<()> {
    let installed: Vec<String> = sqlx::query_scalar("SELECT extname FROM pg_extension")
        .fetch_all(pool)
        .await?;

    let missing: Vec<&String> = required.iter().filter(|e| !installed.contains(e)).collect();

    if !missing.is_empty() {
        let create_commands: Vec<String> = missing
            .iter()
            .map(|e| format!("CREATE EXTENSION IF NOT EXISTS {};", e))
            .collect();

        anyhow::bail!(
            "Missing required PostgreSQL extensions: {:?}\n\
             Please install them with:\n{}",
            missing,
            create_commands.join("\n")
        );
    }

    info!("All required extensions are installed.");
    Ok(())
}

impl Collector {
    /// Create a new Collector.
    ///
    /// * `db_config` – database connection details and required extensions
    /// * `max_queries` – max number of rows to fetch from pg_stat_statements
    pub async fn new(db_config: &DatabaseConfig, max_queries: usize) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(2) // read‑only sidecar
            .connect(&db_config.url)
            .await?;

        // Check that all required extensions exist
        validate_extensions(&pool, &db_config.required_extensions).await?;

        Ok(Self { pool, max_queries })
    }

    /// Takes a snapshot of current workload statistics.
    pub async fn take_snapshot(&self) -> Result<WorkloadSnapshot> {
        let queries = self.collect_queries().await?;
        let tables = self.collect_tables().await?;
        let indexes = self.collect_indexes().await?;

        Ok(WorkloadSnapshot {
            timestamp: Utc::now(),
            database_name: "from_config".into(),
            queries,
            tables,
            indexes,
        })
    }

    async fn collect_queries(&self) -> Result<Vec<QueryStats>> {
        let rows = sqlx::query_as::<_, (i64, String, i64, f64, f64, i64, i64, i64)>(
            r#"
            SELECT queryid, query, calls,
                   total_exec_time::float8, mean_exec_time::float8,
                   rows, shared_blks_hit, shared_blks_read
            FROM pg_stat_statements
            WHERE calls > 0
            ORDER BY total_exec_time DESC
            LIMIT $1
            "#,
        )
            .bind(self.max_queries as i64)
            .fetch_all(&self.pool)
            .await?;

        let stats = rows
            .into_iter()
            .map(|r| QueryStats {
                queryid: r.0,
                query_text: r.1,
                calls: r.2,
                total_exec_time: r.3,
                mean_exec_time: r.4,
                rows: r.5,
                shared_blks_hit: r.6,
                shared_blks_read: r.7,
            })
            .collect();
        Ok(stats)
    }

    async fn collect_tables(&self) -> Result<Vec<TableStats>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64, i64, i64, i64, i64, i64, i64, i64)>(
            r#"
            SELECT schemaname, relname,
                   seq_scan, idx_scan,
                   n_tup_ins, n_tup_upd, n_tup_del,
                   n_live_tup, n_dead_tup,
                   pg_table_size(quote_ident(schemaname)||'.'||quote_ident(relname))::int8 as table_size,
                   pg_indexes_size(quote_ident(schemaname)||'.'||quote_ident(relname))::int8 as indexes_size
            FROM pg_stat_user_tables
            "#,
        )
            .fetch_all(&self.pool)
            .await?;

        let stats = rows
            .into_iter()
            .map(|r| TableStats {
                schema: r.0,
                table_name: r.1,
                seq_scan: r.2,
                idx_scan: r.3,
                n_tup_ins: r.4,
                n_tup_upd: r.5,
                n_tup_del: r.6,
                n_live_tup: r.7,
                n_dead_tup: r.8,
                table_size_bytes: r.9,
                indexes_size_bytes: r.10,
            })
            .collect();
        Ok(stats)
    }

    async fn collect_indexes(&self) -> Result<Vec<IndexInfo>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, bool)>(
            r#"
            SELECT i.schemaname, i.tablename, i.indexname,
                 i.indexdef, pg_idx.indisunique
            FROM pg_indexes i
            JOIN pg_class c ON c.relname = i.indexname
            JOIN pg_index pg_idx ON pg_idx.indexrelid = c.oid
            WHERE i.schemaname NOT IN ('pg_catalog', 'information_schema')
            "#,
        )
            .fetch_all(&self.pool)
            .await?;

        let indexes = rows
            .into_iter()
            .map(|r| {
                let indexdef = r.3;
                let columns = if let Some(start) = indexdef.find('(') {
                    let end = indexdef.rfind(')').unwrap();
                    let col_list = &indexdef[start + 1..end];
                    col_list.split(',').map(|c| c.trim().to_string()).collect()
                } else {
                    vec![]
                };
                IndexInfo {
                    schema: r.0,
                    table_name: r.1,
                    index_name: r.2,
                    columns,
                    is_unique: r.4,
                    index_type: "btree".into(),
                }
            })
            .collect();
        Ok(indexes)
    }
}