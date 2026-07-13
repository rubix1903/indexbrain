use anyhow::Result;          // <-- critical: brings anyhow::Result into scope
use indexbrain_core::{WorkloadSnapshot, QueryStats, TableStats, IndexInfo};
use sqlx::postgres::PgPoolOptions;
// use sqlx::Row;           // not needed if we don't use Row trait directly
use chrono::Utc;
use tracing::info;           // keep if you plan to log, or remove for now
pub struct Collector {
    pool: sqlx::PgPool,
    fetch_query_text: bool,
    max_queries: usize,
}

impl Collector {
    pub async fn new(database_url: &str, fetch_query_text: bool, max_queries: usize) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(2)  // read-only sidecar
            .connect(database_url)
            .await?;
        Ok(Self { pool, fetch_query_text, max_queries })
    }

    /// Takes a snapshot of current workload statistics.
    pub async fn take_snapshot(&self) -> Result<WorkloadSnapshot> {
        let queries = self.collect_queries().await?;
        let tables = self.collect_tables().await?;
        let indexes = self.collect_indexes().await?;

        Ok(WorkloadSnapshot {
            timestamp: Utc::now(),
            database_name: "from_config".into(), // we could get it from DB but keep it simple
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

        let stats = rows.into_iter().map(|r| QueryStats {
            queryid: r.0,
            query_text: r.1,
            calls: r.2,
            total_exec_time: r.3,
            mean_exec_time: r.4,
            rows: r.5,
            shared_blks_hit: r.6,
            shared_blks_read: r.7,
        }).collect();
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
            "#
        )
            .fetch_all(&self.pool)
            .await?;

        let stats = rows.into_iter().map(|r| TableStats {
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
        }).collect();
        Ok(stats)
    }

    async fn collect_indexes(&self) -> Result<Vec<IndexInfo>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, bool)>(
            r#"
            SELECT schemaname, tablename, indexname,
                   indexdef, (indisunique is true)
            FROM pg_indexes
            JOIN pg_index ON pg_indexes.indexname = pg_class.relname
            JOIN pg_class ON pg_class.relname = pg_indexes.indexname
            WHERE pg_indexes.schemaname NOT IN ('pg_catalog','information_schema')
            "#
        )
            .fetch_all(&self.pool)
            .await?;

        let indexes = rows.into_iter().map(|r| {
            // Extract column names from indexdef (simple parser, can be improved)
            let indexdef = r.3;
            let columns = if let Some(start) = indexdef.find('(') {
                let end = indexdef.rfind(')').unwrap();
                let col_list = &indexdef[start+1..end];
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
                index_type: "btree".into(), // we could parse from indexdef
            }
        }).collect();
        Ok(indexes)
    }
}

