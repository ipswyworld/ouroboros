// Migration runner binary - reuses library code
use sqlx::PgPool;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get database URL from environment
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ouro:ouro_pass@127.0.0.1:15432/ouro_db".to_string());

    println!("🔗 Connecting to database...");
    let pool = PgPool::connect(&db_url).await?;

    println!("📊 Running migrations...");

    // Reuse the run_migrations function from the library
    ouro_dag::run_migrations(&pool).await?;

    println!("✅ Migrations completed successfully.");
    Ok(())
}
