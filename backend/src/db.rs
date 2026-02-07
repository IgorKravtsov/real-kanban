use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use std::str::FromStr;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let file_path = if database_url.starts_with("sqlite://") {
        database_url.trim_start_matches("sqlite://")
    } else {
        database_url
    };
    
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent).expect("Failed to create data directory");
    }

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    let schema = include_str!("../schema.sql");
    
    for statement in schema.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        sqlx::raw_sql(statement).execute(&pool).await?;
    }

    run_migrations(&pool).await?;

    tracing::info!("Database initialized successfully");

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let has_sort_order: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name = 'sort_order'"
    )
    .fetch_one(pool)
    .await?;

    if !has_sort_order {
        sqlx::raw_sql("ALTER TABLE projects ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await?;
        
        sqlx::raw_sql(
            "UPDATE projects SET sort_order = id * 1000 WHERE sort_order = 0"
        )
        .execute(pool)
        .await?;
        
        tracing::info!("Migration: Added sort_order column to projects table");
    }

    Ok(())
}
