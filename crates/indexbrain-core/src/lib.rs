pub mod settings;
pub use settings::Settings;
pub mod features;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A snapshot of the database workload at a given moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadSnapshot {
    pub timestamp: DateTime<Utc>,
    pub database_name: String,
    pub queries: Vec<QueryStats>,
    pub tables: Vec<TableStats>,
    pub indexes: Vec<IndexInfo>,
}

/// Statistics for a single query fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub queryid: i64,
    pub query_text: String,
    pub calls: i64,
    pub total_exec_time: f64,    // milliseconds
    pub mean_exec_time: f64,
    pub rows: i64,
    pub shared_blks_hit: i64,
    pub shared_blks_read: i64,
}

/// Table-level access and size stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub schema: String,
    pub table_name: String,
    pub seq_scan: i64,
    pub idx_scan: i64,
    pub n_tup_ins: i64,
    pub n_tup_upd: i64,
    pub n_tup_del: i64,
    pub n_live_tup: i64,
    pub n_dead_tup: i64,
    pub table_size_bytes: i64,
    pub indexes_size_bytes: i64,
}

/// A currently existing index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub schema: String,
    pub table_name: String,
    pub index_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: String,   // btree, hash, etc.
}