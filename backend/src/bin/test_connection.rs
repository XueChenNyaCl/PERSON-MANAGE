use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Testing database connection to: {}", database_url);
    
    // Test connection
    match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Database connection successful!");
            
            // Test simple query
            match sqlx::query("SELECT 1").execute(&pool).await {
                Ok(_) => println!("✅ Test query executed successfully"),
                Err(e) => println!("❌ Test query failed: {}", e),
            }
            
            // Check if _sqlx_migrations table exists
            match sqlx::query("SELECT COUNT(*) FROM _sqlx_migrations")
                .fetch_one(&pool)
                .await
            {
                Ok(row) => {
                    let count: i64 = row.get(0);
                    println!("✅ _sqlx_migrations table exists, count: {}", count);
                    
                    // List applied migrations
                    match sqlx::query("SELECT version, description FROM _sqlx_migrations ORDER BY version")
                        .fetch_all(&pool)
                        .await
                    {
                        Ok(rows) => {
                            println!("Applied migrations:");
                            for row in rows {
                                let version: String = row.get(0);
                                let description: String = row.get(1);
                                println!("  - {}: {}", version, description);
                            }
                        }
                        Err(e) => println!("❌ Failed to list migrations: {}", e),
                    }
                }
                Err(e) => println!("❌ _sqlx_migrations table not found or error: {}", e),
            }
            
            // Try to run migrations
            println!("\nAttempting to run migrations...");
            match sqlx::migrate!().run(&pool).await {
                Ok(_) => println!("✅ Migrations completed successfully"),
                Err(e) => println!("❌ Migrations failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Database connection failed: {}", e);
            println!("Error details: {:?}", e);
        }
    }
    
    Ok(())
}