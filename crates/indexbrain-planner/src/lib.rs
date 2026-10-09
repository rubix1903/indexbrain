use indexbrain_core::{WorkloadSnapshot, Arm};
use regex::Regex;
use tracing::info;

pub fn generate_candidates(
    config: &indexbrain_core::planner::PlannerConfig,
    snapshot: &WorkloadSnapshot,
) -> Vec<Arm> {
    let mut arms = Vec::new();
    let mut column_usage: std::collections::HashMap<(String, String), std::collections::HashSet<String>> = std::collections::HashMap::new();

    // Collect all known table names (schema-qualified) from the snapshot
    let known_tables: std::collections::HashSet<(String, String)> = snapshot
        .tables
        .iter()
        .map(|t| (t.schema.clone(), t.table_name.clone()))
        .collect();

    // Regex to capture table.column references in WHERE/JOIN/ON/GROUP BY/ORDER BY
    let col_re = Regex::new(r#"(?i)(?:WHERE|JOIN|ON|GROUP\s+BY|ORDER\s+BY)\s+.*?([\w.]+)"#).unwrap();

    // **Fix**: First filter all queries that contain any known table, then take the top 10 by total_exec_time
    let mut user_queries: Vec<&indexbrain_core::QueryStats> = snapshot
        .queries
        .iter()
        .filter(|q| {
            let text = &q.query_text;
            known_tables.iter().any(|(schema, table)| {
                text.contains(&format!("{}.{}", schema, table)) || text.contains(table)
            })
        })
        .collect();

    // Sort those by total_exec_time descending and take top-10
    user_queries.sort_by(|a, b| b.total_exec_time.partial_cmp(&a.total_exec_time).unwrap());
    let top_user_queries = user_queries.into_iter().take(10);

    for query in top_user_queries {
        let text = &query.query_text;
        for cap in col_re.captures_iter(text) {
            if let Some(col) = cap.get(1) {
                let full = col.as_str();
                if let Some(dot_pos) = full.find('.') {
                    let table_part = &full[..dot_pos];
                    let column = &full[dot_pos + 1..];
                    // Match to known tables (ignore schema for simplicity)
                    let matched_table = known_tables.iter().find(|(_, t)| t == table_part);
                    if let Some((schema, table)) = matched_table {
                        column_usage
                            .entry((schema.clone(), table.clone()))
                            .or_default()
                            .insert(column.to_string());
                    }
                }
            }
        }
    }

    // Generate index arms from the extracted column usage (same as before)
    for ((schema, table), columns_set) in &column_usage {
        let columns: Vec<_> = columns_set.iter().cloned().collect();
        for col in &columns {
            if arms.len() >= config.max_total_candidates {
                break;
            }
            arms.push(Arm {
                id: arms.len(),
                schema: schema.clone(),
                table_name: table.clone(),
                columns: vec![col.clone()],
                index_type: "btree".into(),
                description: format!("CREATE INDEX ON {}.{} ({});", schema, table, col),
            });
        }
        for i in 0..columns.len() {
            for j in i + 1..columns.len() {
                if arms.len() >= config.max_total_candidates {
                    break;
                }
                arms.push(Arm {
                    id: arms.len(),
                    schema: schema.clone(),
                    table_name: table.clone(),
                    columns: vec![columns[i].clone(), columns[j].clone()],
                    index_type: "btree".into(),
                    description: format!(
                        "CREATE INDEX ON {}.{} ({}, {});",
                        schema, table, columns[i], columns[j]
                    ),
                });
            }
        }
    }

    // Fallback arm
    if arms.is_empty() {
        arms.push(Arm {
            id: 0,
            schema: String::new(),
            table_name: String::new(),
            columns: vec![],
            index_type: "none".into(),
            description: "No candidate indexes found".into(),
        });
    }

    info!("Generated {} candidate index arms", arms.len());
    arms
}