use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub upload: UploadConfig,
    pub app: AppInfoConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub admin_username: String,
    pub admin_password_hash: String, // Argon2 hash
    pub session_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UploadConfig {
    pub max_size_mb: u64,
    pub allowed_extensions: Vec<String>,
    pub upload_dir: PathBuf,
    pub serve_path: String, // e.g., "/uploads"
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppInfoConfig {
    pub brand_name: String,
    pub legal_name: String,
    pub tagline: String,
    pub description: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub social_media: SocialMediaConfig,
    pub seo: SeoConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SocialMediaConfig {
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub youtube: Option<String>,
    pub github: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SeoConfig {
    pub meta_title: String,
    pub meta_description: String,
    pub meta_keywords: String,
    pub og_image: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        
        let mut builder = Config::builder()
            .add_source(File::with_name("config/default").required(true))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(config::Environment::with_prefix("WORKA").separator("_"));

        if let Ok(config_file) = std::env::var("WORKA_CONFIG_FILE") {
            builder = builder.add_source(File::from(std::path::PathBuf::from(config_file)).required(true));
        }

        let mut config: AppConfig = builder.build()?.try_deserialize()?;

        // Allow a plain `PORT` environment variable to override the listening
        // port. This keeps local runs and simple container setups predictable
        // while `WORKA_SERVER_PORT` remains the fully-qualified override.
        if std::env::var("WORKA_SERVER_PORT").is_err() {
            if let Ok(port) = std::env::var("PORT") {
                if let Ok(port) = port.parse::<u16>() {
                    config.server.port = port;
                }
            }
        }

        Ok(config)
    }
}