use salvo::prelude::*;
use crate::state::AppState;
use crate::config::AppConfig;
use crate::models::*;
use crate::views::{AdminHeroView, AdminServiceView, AdminPortfolioView, AdminNewsView, AdminMediaView, AdminSettingView, AdminSettingGroupView};
use crate::state::get_all_settings_map;
use crate::utils::{process_upload, delete_file, generate_slug, ensure_unique_slug};
use crate::middleware::auth::{ADMIN_SESSION_KEY, ADMIN_USERNAME_KEY};
use crate::filters::{self, Or};
use askama::Template;
use std::collections::HashMap;
use serde_json::json;
use chrono::Datelike;

// ============ Admin Templates ============

#[derive(Template)]
#[template(path = "admin/layout.html")]
pub struct AdminLayoutTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct LoginTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub error: String,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct DashboardTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub stats: DashboardStats,
    pub recent_news: Vec<AdminNewsView>,
    pub recent_portfolios: Vec<AdminPortfolioView>,
}

#[derive(Template)]
#[template(path = "admin/services/list.html")]
pub struct ServicesListTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub services: Vec<AdminServiceView>,
}

#[derive(Template)]
#[template(path = "admin/services/form.html")]
pub struct ServiceFormTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub service: AdminServiceView,
    pub is_edit: bool,
    pub media_files: Vec<AdminMediaView>,
}

#[derive(Template)]
#[template(path = "admin/portfolios/list.html")]
pub struct PortfoliosListTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub portfolios: Vec<AdminPortfolioView>,
}

#[derive(Template)]
#[template(path = "admin/portfolios/form.html")]
pub struct PortfolioFormTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub portfolio: AdminPortfolioView,
    pub is_edit: bool,
    pub media_files: Vec<AdminMediaView>,
}

#[derive(Template)]
#[template(path = "admin/news/list.html")]
pub struct NewsListTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub news: Vec<AdminNewsView>,
    pub current_page: i32,
    pub total_pages: i32,
}

#[derive(Template)]
#[template(path = "admin/news/form.html")]
pub struct NewsFormTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub article: AdminNewsView,
    pub is_edit: bool,
    pub media_files: Vec<AdminMediaView>,
}

#[derive(Template)]
#[template(path = "admin/hero/list.html")]
pub struct HeroListTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub heroes: Vec<AdminHeroView>,
    pub media_files: Vec<AdminMediaView>,
}

#[derive(Template)]
#[template(path = "admin/hero/form.html")]
pub struct HeroFormTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub hero: AdminHeroView,
    pub is_edit: bool,
    pub media_files: Vec<AdminMediaView>,
}

#[derive(Template)]
#[template(path = "admin/media/list.html")]
pub struct MediaListTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub media: Vec<AdminMediaView>,
    pub current_page: i32,
    pub total_pages: i32,
}

#[derive(Template)]
#[template(path = "admin/settings/list.html")]
pub struct SettingsListTemplate {
    pub site: SiteInfo,
    pub page: AdminPageInfo,
    pub settings: HashMap<String, String>,
    pub current_user: String,
    pub settings_list: Vec<AdminSettingView>,
    pub groups: Vec<AdminSettingGroupView>,
}

// ============ Admin Handlers ============

#[handler]
pub async fn admin_login_page(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let config = &state.config;
    let db = &state.db;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let page = AdminPageInfo {
        title: "Login Admin".to_string(),
        breadcrumb: vec![BreadcrumbItem { label: "Login".to_string(), url: None }],
    };

    let template = LoginTemplate {
        site,
        page,
        settings,
        error: String::new(),
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_dashboard(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let page = AdminPageInfo {
        title: "Dashboard".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: None },
        ],
    };

    let total_services: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services").fetch_one(db).await.unwrap_or(0);
    let total_portfolios: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portfolios").fetch_one(db).await.unwrap_or(0);
    let total_news: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_articles").fetch_one(db).await.unwrap_or(0);
    let published_news: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_articles WHERE is_published = 1").fetch_one(db).await.unwrap_or(0);
    let total_media: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media_files").fetch_one(db).await.unwrap_or(0);

    let recent_news = sqlx::query_as::<_, NewsArticle>("SELECT * FROM news_articles ORDER BY created_at DESC LIMIT 5")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminNewsView::from)
        .collect();
    let recent_portfolios = sqlx::query_as::<_, Portfolio>("SELECT * FROM portfolios ORDER BY created_at DESC LIMIT 5")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminPortfolioView::from)
        .collect();

    let stats = DashboardStats {
        total_services, total_portfolios, total_news, published_news, total_media,
    };

    let template = DashboardTemplate {
        site,
        page,
        settings,
        current_user: username,
        stats,
        recent_news,
        recent_portfolios,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

// ============ Services Admin ============

#[handler]
pub async fn admin_services_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let services = sqlx::query_as::<_, crate::models::Service>("SELECT * FROM services ORDER BY is_featured DESC, sort_order, created_at DESC")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminServiceView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Kelola Layanan".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Layanan".to_string(), url: None },
        ],
    };

    let template = ServicesListTemplate {
        site, page, settings, current_user: username, services,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_service_new(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Tambah Layanan Baru".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Layanan".to_string(), url: Some("/admin/services".to_string()) },
            BreadcrumbItem { label: "Tambah Baru".to_string(), url: None },
        ],
    };

    let template = ServiceFormTemplate {
        site, page, settings, current_user: username, service: AdminServiceView::default(), is_edit: false, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_service_edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let id = req.param::<String>("id").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let service = sqlx::query_as::<_, crate::models::Service>("SELECT * FROM services WHERE id = ?")
        .bind(&id).fetch_optional(db).await.ok().flatten()
        .map(AdminServiceView::from);

    let service = match service {
        Some(s) => s,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html("Service not found".to_string()));
            return;
        }
    };

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: format!("Edit: {}", service.title),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Layanan".to_string(), url: Some("/admin/services".to_string()) },
            BreadcrumbItem { label: "Edit".to_string(), url: None },
        ],
    };

    let template = ServiceFormTemplate {
        site, page, settings, current_user: username, service, is_edit: true, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_service_save(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let form = req.parse_form::<crate::models::ServiceForm>().await.unwrap_or_default();
    
    let id = req.param::<String>("id");
    let is_new = id.is_none();
    let service_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let slug = if form.slug.is_empty() { generate_slug(&form.title) } else { form.slug };
    
    let existing_slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM services WHERE id != ?")
        .bind(&service_id).fetch_all(db).await.unwrap_or_default();
    let final_slug = ensure_unique_slug(&slug, &existing_slugs);

    let features_json = form.features.map(|f| serde_json::to_string(&f).unwrap_or_default());
    let image_path = if let Some(img_id) = form.image_id {
        sqlx::query_scalar::<_, String>("SELECT path FROM media_files WHERE id = ?")
            .bind(img_id.to_string()).fetch_optional(db).await.ok().flatten()
    } else { None };

    if is_new {
        sqlx::query(
            r#"INSERT INTO services (id, title, slug, short_description, full_description, icon, image_path, category, features, is_active, is_featured, sort_order, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#
        )
        .bind(&service_id)
        .bind(&form.title)
        .bind(&final_slug)
        .bind(&form.short_description)
        .bind(&form.full_description)
        .bind(&form.icon)
        .bind(&image_path)
        .bind(&form.category)
        .bind(&features_json)
        .bind(form.is_active.unwrap_or(true) as i32)
        .bind(form.is_featured.unwrap_or(false) as i32)
        .bind(form.sort_order.unwrap_or(0))
        .execute(db).await.ok();
    } else {
        sqlx::query(
            r#"UPDATE services SET title=?, slug=?, short_description=?, full_description=?, icon=?, image_path=?, category=?, features=?, is_active=?, is_featured=?, sort_order=?, updated_at=datetime('now') WHERE id=?"#
        )
        .bind(&form.title)
        .bind(&final_slug)
        .bind(&form.short_description)
        .bind(&form.full_description)
        .bind(&form.icon)
        .bind(&image_path)
        .bind(&form.category)
        .bind(&features_json)
        .bind(form.is_active.unwrap_or(true) as i32)
        .bind(form.is_featured.unwrap_or(false) as i32)
        .bind(form.sort_order.unwrap_or(0))
        .bind(&service_id)
        .execute(db).await.ok();
    }

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "redirect": "/admin/services",
        "message": if is_new { "Layanan berhasil ditambahkan" } else { "Layanan berhasil diperbarui" }
    }))));
}

#[handler]
pub async fn admin_service_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let id = req.param::<String>("id").unwrap_or_default();

    sqlx::query("DELETE FROM services WHERE id = ?").bind(&id).execute(db).await.ok();

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "Layanan berhasil dihapus"
    }))));
}

// ============ Portfolios Admin ============

#[handler]
pub async fn admin_portfolios_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let portfolios = sqlx::query_as::<_, Portfolio>("SELECT * FROM portfolios ORDER BY is_featured DESC, sort_order, created_at DESC")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminPortfolioView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Kelola Portfolio".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Portfolio".to_string(), url: None },
        ],
    };

    let template = PortfoliosListTemplate {
        site, page, settings, current_user: username, portfolios,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_portfolio_new(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Tambah Portfolio Baru".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Portfolio".to_string(), url: Some("/admin/portfolios".to_string()) },
            BreadcrumbItem { label: "Tambah Baru".to_string(), url: None },
        ],
    };

    let template = PortfolioFormTemplate {
        site, page, settings, current_user: username, portfolio: AdminPortfolioView::default(), is_edit: false, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_portfolio_edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let id = req.param::<String>("id").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let portfolio = sqlx::query_as::<_, Portfolio>("SELECT * FROM portfolios WHERE id = ?")
        .bind(&id).fetch_optional(db).await.ok().flatten()
        .map(AdminPortfolioView::from);

    let portfolio = match portfolio {
        Some(p) => p,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html("Portfolio not found".to_string()));
            return;
        }
    };

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: format!("Edit: {}", portfolio.title),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Portfolio".to_string(), url: Some("/admin/portfolios".to_string()) },
            BreadcrumbItem { label: "Edit".to_string(), url: None },
        ],
    };

    let template = PortfolioFormTemplate {
        site, page, settings, current_user: username, portfolio, is_edit: true, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_portfolio_save(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let form = req.parse_form::<crate::models::PortfolioForm>().await.unwrap_or_default();
    
    let id = req.param::<String>("id");
    let is_new = id.is_none();
    let portfolio_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let slug = if form.slug.is_empty() { generate_slug(&form.title) } else { form.slug };
    
    let existing_slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM portfolios WHERE id != ?")
        .bind(&portfolio_id).fetch_all(db).await.unwrap_or_default();
    let final_slug = ensure_unique_slug(&slug, &existing_slugs);

    let tech_json = form.technologies.map(|t| serde_json::to_string(&t).unwrap_or_default());
    let gallery_json = form.gallery_ids.map(|g| serde_json::to_string(&g.iter().map(|id| id.to_string()).collect::<Vec<_>>()).unwrap_or_default());
    
    let thumbnail_path = if let Some(img_id) = form.thumbnail_id {
        sqlx::query_scalar::<_, String>("SELECT path FROM media_files WHERE id = ?")
            .bind(img_id.to_string()).fetch_optional(db).await.ok().flatten()
    } else { None };

    if is_new {
        sqlx::query(
            r#"INSERT INTO portfolios (id, title, slug, short_description, full_description, client_name, project_url, thumbnail_path, gallery_images, technologies, category, start_date, end_date, is_active, is_featured, sort_order, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#
        )
        .bind(&portfolio_id)
        .bind(&form.title)
        .bind(&final_slug)
        .bind(&form.short_description)
        .bind(&form.full_description)
        .bind(&form.client_name)
        .bind(&form.project_url)
        .bind(&thumbnail_path)
        .bind(&gallery_json)
        .bind(&tech_json)
        .bind(&form.category)
        .bind(&form.start_date.map(|d| d.to_string()))
        .bind(&form.end_date.map(|d| d.to_string()))
        .bind(form.is_active.unwrap_or(true) as i32)
        .bind(form.is_featured.unwrap_or(false) as i32)
        .bind(form.sort_order.unwrap_or(0))
        .execute(db).await.ok();
    } else {
        sqlx::query(
            r#"UPDATE portfolios SET title=?, slug=?, short_description=?, full_description=?, client_name=?, project_url=?, thumbnail_path=?, gallery_images=?, technologies=?, category=?, start_date=?, end_date=?, is_active=?, is_featured=?, sort_order=?, updated_at=datetime('now') WHERE id=?"#
        )
        .bind(&form.title)
        .bind(&final_slug)
        .bind(&form.short_description)
        .bind(&form.full_description)
        .bind(&form.client_name)
        .bind(&form.project_url)
        .bind(&thumbnail_path)
        .bind(&gallery_json)
        .bind(&tech_json)
        .bind(&form.category)
        .bind(&form.start_date.map(|d| d.to_string()))
        .bind(&form.end_date.map(|d| d.to_string()))
        .bind(form.is_active.unwrap_or(true) as i32)
        .bind(form.is_featured.unwrap_or(false) as i32)
        .bind(form.sort_order.unwrap_or(0))
        .bind(&portfolio_id)
        .execute(db).await.ok();
    }

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "redirect": "/admin/portfolios",
        "message": if is_new { "Portfolio berhasil ditambahkan" } else { "Portfolio berhasil diperbarui" }
    }))));
}

#[handler]
pub async fn admin_portfolio_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let id = req.param::<String>("id").unwrap_or_default();

    sqlx::query("DELETE FROM portfolios WHERE id = ?").bind(&id).execute(db).await.ok();

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "Portfolio berhasil dihapus"
    }))));
}

// ============ News Admin ============

#[handler]
pub async fn admin_news_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let page_num = req.query::<i32>("page").unwrap_or(1).max(1);
    let per_page = 20;
    let offset = (page_num - 1) * per_page;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_articles").fetch_one(db).await.unwrap_or(0);
    let news = sqlx::query_as::<_, NewsArticle>("SELECT * FROM news_articles ORDER BY created_at DESC LIMIT ? OFFSET ?")
        .bind(per_page).bind(offset).fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminNewsView::from)
        .collect();

    let total_pages = ((total as f64) / per_page as f64).ceil() as i32;

    let page = AdminPageInfo {
        title: "Kelola Berita & Artikel".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Berita & Artikel".to_string(), url: None },
        ],
    };

    let template = NewsListTemplate {
        site, page, settings, current_user: username, news, current_page: page_num, total_pages,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_news_new(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Tambah Artikel Baru".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Berita & Artikel".to_string(), url: Some("/admin/news".to_string()) },
            BreadcrumbItem { label: "Tambah Baru".to_string(), url: None },
        ],
    };

    let template = NewsFormTemplate {
        site, page, settings, current_user: username, article: AdminNewsView::default(), is_edit: false, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_news_edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let id = req.param::<String>("id").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let article = sqlx::query_as::<_, NewsArticle>("SELECT * FROM news_articles WHERE id = ?")
        .bind(&id).fetch_optional(db).await.ok().flatten()
        .map(AdminNewsView::from);

    let article = match article {
        Some(a) => a,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html("Article not found".to_string()));
            return;
        }
    };

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: format!("Edit: {}", article.title),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Berita & Artikel".to_string(), url: Some("/admin/news".to_string()) },
            BreadcrumbItem { label: "Edit".to_string(), url: None },
        ],
    };

    let template = NewsFormTemplate {
        site, page, settings, current_user: username, article, is_edit: true, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_news_save(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let form = req.parse_form::<crate::models::NewsForm>().await.unwrap_or_default();
    
    let id = req.param::<String>("id");
    let is_new = id.is_none();
    let article_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let slug = if form.slug.is_empty() { generate_slug(&form.title) } else { form.slug };
    
    let existing_slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM news_articles WHERE id != ?")
        .bind(&article_id).fetch_all(db).await.unwrap_or_default();
    let final_slug = ensure_unique_slug(&slug, &existing_slugs);

    let tags_json = form.tags.map(|t| serde_json::to_string(&t).unwrap_or_default());
    let published_at = if form.is_published.unwrap_or(false) { Some(chrono::Utc::now().to_rfc3339()) } else { None };
    
    let thumbnail_path = if let Some(img_id) = form.thumbnail_id {
        sqlx::query_scalar::<_, String>("SELECT path FROM media_files WHERE id = ?")
            .bind(img_id.to_string()).fetch_optional(db).await.ok().flatten()
    } else { None };

    if is_new {
        sqlx::query(
            r#"INSERT INTO news_articles (id, title, slug, excerpt, content, thumbnail_path, author, category, tags, is_published, published_at, meta_title, meta_description, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#
        )
        .bind(&article_id)
        .bind(&form.title)
        .bind(&final_slug)
        .bind(&form.excerpt)
        .bind(&form.content)
        .bind(&thumbnail_path)
        .bind("WORKA Admin")
        .bind(&form.category)
        .bind(&tags_json)
        .bind(form.is_published.unwrap_or(false) as i32)
        .bind(&published_at)
        .bind(&form.meta_title)
        .bind(&form.meta_description)
        .execute(db).await.ok();
    } else {
        sqlx::query(
            r#"UPDATE news_articles SET title=?, slug=?, excerpt=?, content=?, thumbnail_path=?, category=?, tags=?, is_published=?, published_at=?, meta_title=?, meta_description=?, updated_at=datetime('now') WHERE id=?"#
        )
        .bind(&form.title)
        .bind(&final_slug)
        .bind(&form.excerpt)
        .bind(&form.content)
        .bind(&thumbnail_path)
        .bind(&form.category)
        .bind(&tags_json)
        .bind(form.is_published.unwrap_or(false) as i32)
        .bind(&published_at)
        .bind(&form.meta_title)
        .bind(&form.meta_description)
        .bind(&article_id)
        .execute(db).await.ok();
    }

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "redirect": "/admin/news",
        "message": if is_new { "Artikel berhasil ditambahkan" } else { "Artikel berhasil diperbarui" }
    }))));
}

#[handler]
pub async fn admin_news_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let id = req.param::<String>("id").unwrap_or_default();

    sqlx::query("DELETE FROM news_articles WHERE id = ?").bind(&id).execute(db).await.ok();

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "Artikel berhasil dihapus"
    }))));
}

// ============ Hero Banner Admin ============

#[handler]
pub async fn admin_hero_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let heroes = sqlx::query_as::<_, HeroBanner>("SELECT * FROM hero_banners ORDER BY sort_order, created_at DESC")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminHeroView::from)
        .collect();
    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Kelola Hero Banner".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Hero Banner".to_string(), url: None },
        ],
    };

    let template = HeroListTemplate {
        site, page, settings, current_user: username, heroes, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_hero_new(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Tambah Hero Banner".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Hero Banner".to_string(), url: Some("/admin/hero".to_string()) },
            BreadcrumbItem { label: "Tambah Baru".to_string(), url: None },
        ],
    };

    let template = HeroFormTemplate {
        site, page, settings, current_user: username, hero: AdminHeroView::default(), is_edit: false, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_hero_edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let id = req.param::<String>("id").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let hero = sqlx::query_as::<_, HeroBanner>("SELECT * FROM hero_banners WHERE id = ?")
        .bind(&id).fetch_optional(db).await.ok().flatten()
        .map(AdminHeroView::from);

    let hero = match hero {
        Some(h) => h,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html("Hero banner not found".to_string()));
            return;
        }
    };

    let media_files = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let page = AdminPageInfo {
        title: "Edit Hero Banner".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Hero Banner".to_string(), url: Some("/admin/hero".to_string()) },
            BreadcrumbItem { label: "Edit".to_string(), url: None },
        ],
    };

    let template = HeroFormTemplate {
        site, page, settings, current_user: username, hero, is_edit: true, media_files,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_hero_save(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let form = req.parse_form::<crate::models::HeroBannerForm>().await.unwrap_or_default();
    
    let id = req.param::<String>("id");
    let is_new = id.is_none();
    let hero_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let image_path = if let Some(img_id) = form.image_id {
        sqlx::query_scalar::<_, String>("SELECT path FROM media_files WHERE id = ?")
            .bind(img_id.to_string()).fetch_optional(&state.db).await.ok().flatten()
    } else { None };

    if is_new {
        sqlx::query(
            r#"INSERT INTO hero_banners (id, title, subtitle, description, cta_text, cta_link, image_path, background_color, text_color, is_active, sort_order, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#
        )
        .bind(&hero_id)
        .bind(&form.title)
        .bind(&form.subtitle)
        .bind(&form.description)
        .bind(&form.cta_text)
        .bind(&form.cta_link)
        .bind(&image_path)
        .bind(&form.background_color)
        .bind(&form.text_color)
        .bind(form.is_active.unwrap_or(true) as i32)
        .bind(form.sort_order.unwrap_or(0))
        .execute(db).await.ok();
    } else {
        sqlx::query(
            r#"UPDATE hero_banners SET title=?, subtitle=?, description=?, cta_text=?, cta_link=?, image_path=?, background_color=?, text_color=?, is_active=?, sort_order=?, updated_at=datetime('now') WHERE id=?"#
        )
        .bind(&form.title)
        .bind(&form.subtitle)
        .bind(&form.description)
        .bind(&form.cta_text)
        .bind(&form.cta_link)
        .bind(&image_path)
        .bind(&form.background_color)
        .bind(&form.text_color)
        .bind(form.is_active.unwrap_or(true) as i32)
        .bind(form.sort_order.unwrap_or(0))
        .bind(&hero_id)
        .execute(db).await.ok();
    }

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "redirect": "/admin/hero",
        "message": if is_new { "Hero banner berhasil ditambahkan" } else { "Hero banner berhasil diperbarui" }
    }))));
}

#[handler]
pub async fn admin_hero_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let id = req.param::<String>("id").unwrap_or_default();

    sqlx::query("DELETE FROM hero_banners WHERE id = ?").bind(&id).execute(db).await.ok();

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "Hero banner berhasil dihapus"
    }))));
}

// ============ Media Admin ============

#[handler]
pub async fn admin_media_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let page_num = req.query::<i32>("page").unwrap_or(1).max(1);
    let per_page = 30;
    let offset = (page_num - 1) * per_page;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media_files").fetch_one(db).await.unwrap_or(0);
    let media = sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files ORDER BY created_at DESC LIMIT ? OFFSET ?")
        .bind(per_page).bind(offset).fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminMediaView::from)
        .collect();

    let total_pages = ((total as f64) / per_page as f64).ceil() as i32;

    let page = AdminPageInfo {
        title: "Kelola Media".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Media".to_string(), url: None },
        ],
    };

    let template = MediaListTemplate {
        site, page, settings, current_user: username, media, current_page: page_num, total_pages,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_media_upload(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let config = &state.config;
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let files = req.files("files").await;
    let mut uploaded = Vec::new();

    if let Some(file_vec) = files {
        for file in file_vec {
            if let Ok(data) = tokio::fs::read(file.path()).await {
                let original_name = file.name().unwrap_or("upload").to_string();
                match process_upload(data, original_name, config, Some(username.clone())).await {
                    Ok(media) => {
                        sqlx::query(
                            r#"INSERT INTO media_files (id, filename, original_name, mime_type, size_bytes, width, height, path, alt_text, uploaded_by, created_at)
                               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"#
                        )
                        .bind(media.id.to_string())
                        .bind(&media.filename)
                        .bind(&media.original_name)
                        .bind(&media.mime_type)
                        .bind(media.size_bytes)
                        .bind(media.width)
                        .bind(media.height)
                        .bind(&media.path)
                        .bind(&media.alt_text)
                        .bind(&media.uploaded_by)
                        .execute(&state.db).await.ok();
                        uploaded.push(media);
                    }
                    Err(e) => {
                        tracing::error!("Upload failed: {}", e);
                    }
                }
            }
        }
    }

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "uploaded": uploaded,
        "message": format!("{} file berhasil diupload", uploaded.len())
    }))));
}

#[handler]
pub async fn admin_media_delete(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let id = req.param::<String>("id").unwrap_or_default();

    let path: Option<String> = sqlx::query_scalar::<_, String>("SELECT path FROM media_files WHERE id = ?")
        .bind(&id).fetch_optional(db).await.ok().flatten();

    if let Some(p) = path {
        delete_file(&p, config).await.ok();
    }

    sqlx::query("DELETE FROM media_files WHERE id = ?").bind(&id).execute(db).await.ok();

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "File berhasil dihapus"
    }))));
}

// ============ Settings Admin ============

#[handler]
pub async fn admin_settings_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_admin_site_info(config, &settings);
    let username = depot
        .session()
        .and_then(|session| session.get::<String>(ADMIN_USERNAME_KEY))
        .unwrap_or_default();

    let settings_list: Vec<AdminSettingView> = sqlx::query_as::<_, Setting>("SELECT * FROM settings ORDER BY \"group\", key")
        .fetch_all(db).await.unwrap_or_default()
        .into_iter()
        .map(AdminSettingView::from)
        .collect();

    let group_meta = |key: &str| -> (&'static str, &'static str) {
        match key {
            "general" => ("Umum", "layout-dashboard"),
            "contact" => ("Kontak", "phone"),
            "social" => ("Media Sosial", "share-2"),
            "seo" => ("SEO", "search"),
            "appearance" => ("Tampilan", "palette"),
            _ => ("Lainnya", "settings"),
        }
    };

    let mut group_keys: Vec<String> = Vec::new();
    for setting in &settings_list {
        if !group_keys.contains(&setting.group) {
            group_keys.push(setting.group.clone());
        }
    }

    let groups: Vec<AdminSettingGroupView> = group_keys
        .into_iter()
        .map(|key| {
            let (name, icon) = group_meta(&key);
            let items = settings_list
                .iter()
                .filter(|s| s.group == key)
                .cloned()
                .collect();
            AdminSettingGroupView {
                key,
                name: name.to_string(),
                icon: icon.to_string(),
                settings: items,
            }
        })
        .collect();

    let page = AdminPageInfo {
        title: "Pengaturan Website".to_string(),
        breadcrumb: vec![
            BreadcrumbItem { label: "Dashboard".to_string(), url: Some("/admin/dashboard".to_string()) },
            BreadcrumbItem { label: "Pengaturan".to_string(), url: None },
        ],
    };

    let template = SettingsListTemplate {
        site, page, settings, current_user: username, settings_list, groups,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn admin_settings_save(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    
    let form_data = req
        .parse_form::<HashMap<String, String>>()
        .await
        .unwrap_or_default();

    for (key, value) in form_data {
        sqlx::query("UPDATE settings SET value = ?, updated_at = datetime('now') WHERE key = ?")
            .bind(&value)
            .bind(&key)
            .execute(db).await.ok();
    }

    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "Pengaturan berhasil disimpan"
    }))));
}

// ============ Helpers ============

fn setting_value(settings: &HashMap<String, String>, key: &str, default: &str) -> String {
    settings
        .get(key)
        .map(|value| value.trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn build_admin_site_info(config: &AppConfig, settings: &HashMap<String, String>) -> SiteInfo {
    SiteInfo {
        brand_name: setting_value(settings, "site_brand_name", &config.app.brand_name),
        legal_name: setting_value(settings, "site_legal_name", &config.app.legal_name),
        tagline: setting_value(settings, "site_tagline", &config.app.tagline),
        description: setting_value(settings, "site_description", &config.app.description),
        address: setting_value(settings, "site_address", &config.app.address),
        phone: setting_value(settings, "site_phone", &config.app.phone),
        email: setting_value(settings, "site_email", &config.app.email),
        website: setting_value(settings, "site_website", &config.app.website),
        logo: setting_value(settings, "site_logo", ""),
        primary_color: setting_value(settings, "primary_color", "#3b82f6"),
        secondary_color: setting_value(settings, "secondary_color", "#64748b"),
        hero_background_color: setting_value(settings, "hero_background_color", "#0f172a"),
        hero_text_color: setting_value(settings, "hero_text_color", "#ffffff"),
        social_media: SocialMediaInfo {
            instagram: setting_value(settings, "social_instagram", config.app.social_media.instagram.as_deref().unwrap_or("")),
            linkedin: setting_value(settings, "social_linkedin", config.app.social_media.linkedin.as_deref().unwrap_or("")),
            twitter: setting_value(settings, "social_twitter", config.app.social_media.twitter.as_deref().unwrap_or("")),
            youtube: setting_value(settings, "social_youtube", config.app.social_media.youtube.as_deref().unwrap_or("")),
            github: setting_value(settings, "social_github", config.app.social_media.github.as_deref().unwrap_or("")),
        },
        seo: SeoInfo {
            meta_title: setting_value(settings, "seo_meta_title", &config.app.seo.meta_title),
            meta_description: setting_value(settings, "seo_meta_description", &config.app.seo.meta_description),
            meta_keywords: setting_value(settings, "seo_meta_keywords", &config.app.seo.meta_keywords),
            og_image: setting_value(settings, "seo_og_image", &config.app.seo.og_image),
        },
        current_year: chrono::Utc::now().year(),
    }
}