// tests/db_smoke.rs
use sqlx::PgPool;
use std::env;

#[tokio::test]
async fn db_connect_and_simple_query() {
    let db_url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("DATABASE_URL not set. Skipping db_smoke test.");
            return;
        }
    };
    let pool = PgPool::connect(&db_url).await.expect("connect pg");
    // run a simple read; migrations must have been applied
    let row: (i64,) = sqlx::query_as("SELECT 1 as one")
        .fetch_one(&pool).await.expect("select 1");
    assert_eq!(row.0, 1);
}
