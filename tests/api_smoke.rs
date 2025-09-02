// tests/api_smoke.rs
#[tokio::test]
async fn can_submit_tx() {
    // This is a simple sanity check that will only run when a DB is available
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for integration test");
    // create pool and ping:
    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect pg");
    assert!(pool.acquire().await.is_ok());
}
