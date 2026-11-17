use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;
use crate::config::AppConfig;
use crate::models::AdminUser;
use anyhow::Result;

pub type DbPool = SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let config = Arc::new(config);
        
        let db = SqlitePoolOptions::new()
            .max_connections(config.database.max_connections)
            .connect(&config.database.url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&db).await?;

        // Ensure default admin exists
        Self::ensure_default_admin(&db, &config).await?;

        Ok(Self { db, config })
    }

    async fn ensure_default_admin(db: &DbPool, config: &AppConfig) -> Result<()> {
        let exists: Option<AdminUser> = sqlx::query_as(
            "SELECT * FROM admin_users WHERE username = ?"
        )
        .bind(&config.auth.admin_username)
        .fetch_optional(db)
        .await?;

        if exists.is_none() {
            // Hash the password from config
            let password_hash = &config.auth.admin_password_hash;
            sqlx::query(
                r#"
                INSERT INTO admin_users (id, username, password_hash, full_name, email, is_active, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, 1, datetime('now'), datetime('now'))
                "#
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&config.auth.admin_username)
            .bind(password_hash)
            .bind("WORKA Admin")
            .bind("admin@worka.solution")
            .execute(db)
            .await?;
            
            tracing::info!("Default admin user created");
        }

        Ok(())
    }
}

// Helper to get setting value
pub async fn get_setting(db: &DbPool, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn get_settings_group(db: &DbPool, group: &str) -> Result<Vec<(String, String)>> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings WHERE \"group\" = ? ORDER BY key"
    )
    .bind(group)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

pub async fn get_all_settings_map(db: &DbPool) -> Result<std::collections::HashMap<String, String>> {
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM settings")
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().collect())
}