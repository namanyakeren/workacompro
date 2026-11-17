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

        // The config crate treats `_` as a nesting separator, so multi-word
        // keys such as `server.base_url` or `app.brand_name` cannot be
        // overridden through their documented WORKA_* variables (e.g.
        // WORKA_APP_BRAND_NAME would become `app.brand.name`). Apply those
        // overrides explicitly so docker-compose settings take effect.
        fn apply_env<T: std::str::FromStr>(key: &str, target: &mut T) {
            if let Ok(raw) = std::env::var(key) {
                if let Ok(value) = raw.parse::<T>() {
                    *target = value;
                }
            }
        }
        fn apply_opt_env(key: &str, target: &mut Option<String>) {
            if let Ok(value) = std::env::var(key) {
                *target = Some(value);
            }
        }

        apply_env("WORKA_SERVER_BASE_URL", &mut config.server.base_url);
        apply_env("WORKA_DATABASE_MAX_CONNECTIONS", &mut config.database.max_connections);
        apply_env("WORKA_AUTH_ADMIN_USERNAME", &mut config.auth.admin_username);
        apply_env("WORKA_AUTH_SESSION_SECRET", &mut config.auth.session_secret);
        apply_env("WORKA_UPLOAD_MAX_SIZE_MB", &mut config.upload.max_size_mb);
        apply_env("WORKA_UPLOAD_UPLOAD_DIR", &mut config.upload.upload_dir);
        apply_env("WORKA_UPLOAD_SERVE_PATH", &mut config.upload.serve_path);
        if let Ok(raw) = std::env::var("WORKA_UPLOAD_ALLOWED_EXTENSIONS") {
            let extensions: Vec<String> = raw
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect();
            if !extensions.is_empty() {
                config.upload.allowed_extensions = extensions;
            }
        }

        apply_env("WORKA_APP_BRAND_NAME", &mut config.app.brand_name);
        apply_env("WORKA_APP_LEGAL_NAME", &mut config.app.legal_name);
        apply_env("WORKA_APP_TAGLINE", &mut config.app.tagline);
        apply_env("WORKA_APP_DESCRIPTION", &mut config.app.description);
        apply_env("WORKA_APP_ADDRESS", &mut config.app.address);
        apply_env("WORKA_APP_PHONE", &mut config.app.phone);
        apply_env("WORKA_APP_EMAIL", &mut config.app.email);
        apply_env("WORKA_APP_WEBSITE", &mut config.app.website);
        apply_opt_env("WORKA_APP_SOCIAL_MEDIA_INSTAGRAM", &mut config.app.social_media.instagram);
        apply_opt_env("WORKA_APP_SOCIAL_MEDIA_LINKEDIN", &mut config.app.social_media.linkedin);
        apply_opt_env("WORKA_APP_SOCIAL_MEDIA_TWITTER", &mut config.app.social_media.twitter);
        apply_opt_env("WORKA_APP_SOCIAL_MEDIA_YOUTUBE", &mut config.app.social_media.youtube);
        apply_opt_env("WORKA_APP_SOCIAL_MEDIA_GITHUB", &mut config.app.social_media.github);
        apply_env("WORKA_APP_SEO_META_TITLE", &mut config.app.seo.meta_title);
        apply_env("WORKA_APP_SEO_META_DESCRIPTION", &mut config.app.seo.meta_description);
        apply_env("WORKA_APP_SEO_META_KEYWORDS", &mut config.app.seo.meta_keywords);
        apply_env("WORKA_APP_SEO_OG_IMAGE", &mut config.app.seo.og_image);

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