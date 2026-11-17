use salvo::prelude::*;
use salvo::session::{SessionHandler, MemoryStore};
use salvo::cors::Cors;
use salvo::compression::Compression;
use salvo::http::Method;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod filters;
mod handlers;
mod middleware;
mod models;
mod state;
mod utils;
mod views;

use config::AppConfig;
use handlers::{admin, public};
use middleware::auth::{auth_middleware, login_handler, logout_handler, optional_auth};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,worka_solution=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    
    // Create upload directory
    tokio::fs::create_dir_all(&config.upload.upload_dir).await?;
    
    // Initialize database and app state
    let app_state = AppState::new(config.clone()).await?;
    
    tracing::info!("Database connected and migrations applied");
    tracing::info!("Server starting on {}:{}", config.server.host, config.server.port);

    // Session store
    let session_store = MemoryStore::new();
    let session_handler = SessionHandler::builder(
        session_store,
        config.auth.session_secret.as_bytes(),
    )
    .build()
    .unwrap();

    // Build router
    let router = Router::new()
        .hoop(Compression::new())
        .hoop(Cors::new().allow_origin("*").allow_headers("*").allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS]).into_handler())
        .hoop(session_handler)
        .hoop(salvo::affix::inject(app_state))
        .hoop(optional_auth)
        // Public routes
        .push(Router::with_path("/").get(public::home_handler))
        .push(Router::with_path("/services").get(public::services_handler))
        .push(Router::with_path("/services/<slug>").get(public::service_detail_handler))
        .push(Router::with_path("/portfolio").get(public::portfolio_handler))
        .push(Router::with_path("/portfolio/<slug>").get(public::portfolio_detail_handler))
        .push(Router::with_path("/news").get(public::news_handler))
        .push(Router::with_path("/news/<slug>").get(public::news_detail_handler))
        .push(Router::with_path("/about").get(public::about_handler))
        .push(Router::with_path("/contact").get(public::contact_handler))
        .push(Router::with_path("/contact").post(public::contact_submit_handler))
        .push(Router::with_path("/api/contact").post(public::contact_submit_handler))
        .push(Router::with_path("/newsletter").post(public::newsletter_submit_handler))
        // Static files
        .push(Router::with_path("/static/<**path>").get(StaticFile::new("static/")))
        .push(Router::with_path("/css/<**path>").get(StaticFile::new("static/css/")))
        .push(Router::with_path("/uploads/<**path>").get(StaticFile::new(config.upload.upload_dir.clone())))
        .push(Router::with_path("/images/<**path>").get(StaticFile::new("static/images/")))
        // Admin routes
        .push(
            Router::with_path("/admin/login")
                .get(admin::admin_login_page)
                .post(login_handler)
        )
        .push(
            Router::with_path("/admin/logout")
                .get(logout_handler)
        )
        .push(
            Router::with_path("/admin")
                .hoop(auth_middleware)
                .push(Router::with_path("/dashboard").get(admin::admin_dashboard))
                // Services
                .push(Router::with_path("/services").get(admin::admin_services_list))
                .push(Router::with_path("/services/new").get(admin::admin_service_new))
                .push(Router::with_path("/services/<id>/edit").get(admin::admin_service_edit))
                .push(Router::with_path("/services/<id>/save").post(admin::admin_service_save))
                .push(Router::with_path("/services").post(admin::admin_service_save))
                .push(Router::with_path("/services/<id>").delete(admin::admin_service_delete))
                // Portfolios
                .push(Router::with_path("/portfolios").get(admin::admin_portfolios_list))
                .push(Router::with_path("/portfolios/new").get(admin::admin_portfolio_new))
                .push(Router::with_path("/portfolios/<id>/edit").get(admin::admin_portfolio_edit))
                .push(Router::with_path("/portfolios/<id>/save").post(admin::admin_portfolio_save))
                .push(Router::with_path("/portfolios").post(admin::admin_portfolio_save))
                .push(Router::with_path("/portfolios/<id>").delete(admin::admin_portfolio_delete))
                // News
                .push(Router::with_path("/news").get(admin::admin_news_list))
                .push(Router::with_path("/news/new").get(admin::admin_news_new))
                .push(Router::with_path("/news/<id>/edit").get(admin::admin_news_edit))
                .push(Router::with_path("/news/<id>/save").post(admin::admin_news_save))
                .push(Router::with_path("/news").post(admin::admin_news_save))
                .push(Router::with_path("/news/<id>").delete(admin::admin_news_delete))
                // Hero
                .push(Router::with_path("/hero").get(admin::admin_hero_list))
                .push(Router::with_path("/hero/new").get(admin::admin_hero_new))
                .push(Router::with_path("/hero/<id>/edit").get(admin::admin_hero_edit))
                .push(Router::with_path("/hero/<id>/save").post(admin::admin_hero_save))
                .push(Router::with_path("/hero").post(admin::admin_hero_save))
                .push(Router::with_path("/hero/<id>").delete(admin::admin_hero_delete))
                // Media
                .push(Router::with_path("/media").get(admin::admin_media_list))
                .push(Router::with_path("/media/upload").post(admin::admin_media_upload))
                .push(Router::with_path("/media/<id>").delete(admin::admin_media_delete))
                // Settings
                .push(Router::with_path("/settings").get(admin::admin_settings_list))
                .push(Router::with_path("/settings/save").post(admin::admin_settings_save))
        );

    // Create acceptor
    let acceptor = TcpListener::new(format!("{}:{}", config.server.host, config.server.port)).bind().await;
    
    // Serve
    Server::new(acceptor).serve(router).await;

    Ok(())
}