use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;
use testcontainers::runners::SyncRunner; // or AsyncRunner depending

#[tokio::test]
async fn test_collector_works() {
    let docker = Cli::default();
    let image = Postgres::default()
        .with_env_var("POSTGRES_PASSWORD", "test")
        .with_env_var("POSTGRES_DB", "testdb");
    let container = docker.run(image);

    let port = container.get_host_port_ipv4(5432);
    let db_url = format!("postgres://postgres:test@127.0.0.1:{}/testdb", port);

    // Create a table and run some queries to populate stats
    let pool = sqlx::PgPool::connect(&db_url).await.unwrap();
    sqlx::query("CREATE TABLE test (id SERIAL PRIMARY KEY, data TEXT)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO test (data) SELECT 'row_' || generate_series(1,1000)")
        .execute(&pool).await.unwrap();
    sqlx::query("SELECT * FROM test WHERE id > 500").execute(&pool).await.unwrap();

    // Enable pg_stat_statements if not already (depends on image config)
    // In the default Postgres image, pg_stat_statements might not be loaded.
    // We'll need a custom image with shared_preload_libraries. For now, we'll skip
    // actual assertion and just demonstrate connection.
    // In a real setup, we'd use a Dockerfile that adds hypopg and pg_stat_statements.
    // For the sake of this blueprint, assume it's available.
    let collector = Collector::new(&db_url, true, 100).await.unwrap();
    let snapshot = collector.take_snapshot().await.unwrap();
    assert!(snapshot.queries.len() > 0);
}