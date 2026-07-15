use indexbrain_core::{WorkloadSnapshot, Arm};
use regex::Regex;
use tracing::info;

pub fn generate_candidates(
    config: &indexbrain_core::planner::PlannerConfig,
    snapshot: &WorkloadSnapshot,
) -> Vec<Arm> {
    let mut arms = Vec::new();
    let mut column_usage: std::collections::HashMap<(String, String), std::collections::HashSet<String>> = std::collections::HashMap::new();
    let col_re = Regex::new(r#"(?i)(?:WHERE|JOIN|ON|GROUP\s+BY|ORDER\s+BY)\s+.*?([\w.]+)"#).unwrap();

    let mut queries: Vec<_> = snapshot.queries.iter().collect();
    queries.sort_by(|a, b| b.total_exec_time.partial_cmp(&a.total_exec_time).unwrap());
    for query in queries.iter().take(10) {
        for cap in col_re.captures_iter(&query.query_text) {
            if let Some(col) = cap.get(1) {
                let full = col.as_str();
                if let Some(dot_pos) = full.find('.') {
                    let table = &full[..dot_pos];
                    let column = &full[dot_pos+1..];
                    column_usage.entry(("public".to_string(), table.to_string()))
                        .or_default().insert(column.to_string());
                }
            }
        }
    }

    for ((schema, table), columns_set) in &column_usage {
        let columns: Vec<_> = columns_set.iter().cloned().collect();
        for col in &columns {
            if arms.len() >= config.max_total_candidates { break; }
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
            for j in i+1..columns.len() {
                if arms.len() >= config.max_total_candidates { break; }
                arms.push(Arm {
                    id: arms.len(),
                    schema: schema.clone(),
                    table_name: table.clone(),
                    columns: vec![columns[i].clone(), columns[j].clone()],
                    index_type: "btree".into(),
                    description: format!("CREATE INDEX ON {}.{} ({}, {});", schema, table, columns[i], columns[j]),
                });
            }
        }
    }

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