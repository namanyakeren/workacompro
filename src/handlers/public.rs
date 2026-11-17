use salvo::prelude::*;
use crate::state::AppState;
use crate::config::AppConfig;
use crate::models::*;
use crate::views::{HeroView, ServiceView, PortfolioView, NewsView};
use crate::state::get_all_settings_map;
use crate::utils::{generate_slug, ensure_unique_slug};
use crate::filters::{self, Or};
use askama::Template;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use chrono::Datelike;

// ============ Templates ============

#[derive(Template)]
#[template(path = "public/layout.html")]
pub struct LayoutTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub content: String,
}

#[derive(Template)]
#[template(path = "public/home.html")]
pub struct HomeTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub hero: HeroView,
    pub services: Vec<ServiceView>,
    pub featured_portfolios: Vec<PortfolioView>,
    pub latest_news: Vec<NewsView>,
}

#[derive(Template)]
#[template(path = "public/services.html")]
pub struct ServicesTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub services: Vec<ServiceView>,
    pub categories: Vec<String>,
}

#[derive(Template)]
#[template(path = "public/service_detail.html")]
pub struct ServiceDetailTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub service: ServiceView,
    pub related_services: Vec<ServiceView>,
}

#[derive(Template)]
#[template(path = "public/portfolio.html")]
pub struct PortfolioTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub portfolios: Vec<PortfolioView>,
    pub categories: Vec<String>,
}

#[derive(Template)]
#[template(path = "public/portfolio_detail.html")]
pub struct PortfolioDetailTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub portfolio: PortfolioView,
    pub related_portfolios: Vec<PortfolioView>,
}

#[derive(Template)]
#[template(path = "public/news.html")]
pub struct NewsTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub news: Vec<NewsView>,
    pub categories: Vec<String>,
    pub current_page: i32,
    pub total_pages: i32,
}

#[derive(Template)]
#[template(path = "public/news_detail.html")]
pub struct NewsDetailTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub article: NewsView,
    pub related_news: Vec<NewsView>,
}

#[derive(Template)]
#[template(path = "public/about.html")]
pub struct AboutTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
    pub team: Vec<TeamMember>,
    pub klbi_codes: Vec<KLBICode>,
}

#[derive(Template)]
#[template(path = "public/contact.html")]
pub struct ContactTemplate {
    pub site: SiteInfo,
    pub page: PageInfo,
    pub settings: HashMap<String, String>,
}

// ============ Public Handlers ============

#[handler]
pub async fn home_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);
    let page = PageInfo {
        title: format!("{} - {}", config.app.brand_name, config.app.tagline),
        description: config.app.description.clone(),
        url: config.server.base_url.clone(),
        og_image: format!("{}{}", config.server.base_url, settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string())),
    };

    let hero = sqlx::query_as::<_, HeroBanner>(
        "SELECT * FROM hero_banners WHERE is_active = 1 ORDER BY sort_order LIMIT 1"
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .map(HeroView::from)
    .unwrap_or_default();

    let services = sqlx::query_as::<_, crate::models::Service>(
        "SELECT * FROM services WHERE is_active = 1 ORDER BY is_featured DESC, sort_order LIMIT 6"
    )
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(ServiceView::from)
    .collect();

    let featured_portfolios = sqlx::query_as::<_, Portfolio>(
        "SELECT * FROM portfolios WHERE is_active = 1 AND is_featured = 1 ORDER BY sort_order LIMIT 6"
    )
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(PortfolioView::from)
    .collect();

    let latest_news = sqlx::query_as::<_, NewsArticle>(
        "SELECT * FROM news_articles WHERE is_published = 1 ORDER BY published_at DESC LIMIT 3"
    )
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(NewsView::from)
    .collect();

    let template = HomeTemplate {
        site,
        page,
        settings,
        hero,
        services,
        featured_portfolios,
        latest_news,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn services_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);
    let page = PageInfo {
        title: format!("Layanan - {}", config.app.brand_name),
        description: "Layanan teknologi terintegrasi: software development, IT consulting, desain grafis, jaringan & keamanan, hardware repair".to_string(),
        url: format!("{}/services", config.server.base_url),
        og_image: format!("{}{}", config.server.base_url, settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string())),
    };

    let services: Vec<ServiceView> = sqlx::query_as::<_, crate::models::Service>(
        "SELECT * FROM services WHERE is_active = 1 ORDER BY is_featured DESC, sort_order"
    )
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(ServiceView::from)
    .collect();

    let categories: Vec<String> = services.iter()
        .filter_map(|s| if s.category.is_empty() { None } else { Some(s.category.clone()) })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let template = ServicesTemplate {
        site,
        page,
        settings,
        services,
        categories,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn service_detail_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let slug = req.param::<String>("slug").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);

    let service = sqlx::query_as::<_, crate::models::Service>(
        "SELECT * FROM services WHERE slug = ? AND is_active = 1"
    )
    .bind(&slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .map(ServiceView::from);

    let service = match service {
        Some(s) => s,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html(render_404(&site, &settings).await));
            return;
        }
    };

    let page = PageInfo {
        title: format!("{} - {}", service.title, config.app.brand_name),
        description: service.short_description.clone(),
        url: format!("{}/services/{}", config.server.base_url, slug),
        og_image: format!("{}{}", config.server.base_url, if service.image_path.is_empty() { settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string()) } else { service.image_path.clone() }),
    };

    let related_services = sqlx::query_as::<_, crate::models::Service>(
        "SELECT * FROM services WHERE is_active = 1 AND id != ? ORDER BY is_featured DESC, sort_order LIMIT 3"
    )
    .bind(&service.id)
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(ServiceView::from)
    .collect();

    let template = ServiceDetailTemplate {
        site,
        page,
        settings,
        service,
        related_services,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn portfolio_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);
    let page = PageInfo {
        title: format!("Portfolio - {}", config.app.brand_name),
        description: "Portfolio proyek kami: pengembangan software, desain grafis, infrastruktur IT, dan solusi teknologi lainnya".to_string(),
        url: format!("{}/portfolio", config.server.base_url),
        og_image: format!("{}{}", config.server.base_url, settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string())),
    };

    let portfolios: Vec<PortfolioView> = sqlx::query_as::<_, Portfolio>(
        "SELECT * FROM portfolios WHERE is_active = 1 ORDER BY is_featured DESC, sort_order"
    )
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(PortfolioView::from)
    .collect();

    let categories: Vec<String> = portfolios.iter()
        .filter_map(|p| if p.category.is_empty() { None } else { Some(p.category.clone()) })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let template = PortfolioTemplate {
        site,
        page,
        settings,
        portfolios,
        categories,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn portfolio_detail_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let slug = req.param::<String>("slug").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);

    let portfolio = sqlx::query_as::<_, Portfolio>(
        "SELECT * FROM portfolios WHERE slug = ? AND is_active = 1"
    )
    .bind(&slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .map(PortfolioView::from);

    let portfolio = match portfolio {
        Some(p) => p,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html(render_404(&site, &settings).await));
            return;
        }
    };

    let page = PageInfo {
        title: format!("{} - {}", portfolio.title, config.app.brand_name),
        description: portfolio.short_description.clone(),
        url: format!("{}/portfolio/{}", config.server.base_url, slug),
        og_image: format!("{}{}", config.server.base_url, if portfolio.thumbnail_path.is_empty() { settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string()) } else { portfolio.thumbnail_path.clone() }),
    };

    let related_portfolios = sqlx::query_as::<_, Portfolio>(
        "SELECT * FROM portfolios WHERE is_active = 1 AND id != ? ORDER BY is_featured DESC, sort_order LIMIT 3"
    )
    .bind(&portfolio.id)
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(PortfolioView::from)
    .collect();

    let template = PortfolioDetailTemplate {
        site,
        page,
        settings,
        portfolio,
        related_portfolios,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn news_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let page_num = req.query::<i32>("page").unwrap_or(1).max(1);
    let per_page = 9;
    let offset = (page_num - 1) * per_page;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);
    let page = PageInfo {
        title: format!("Berita & Artikel - {}", config.app.brand_name),
        description: "Berita terbaru, artikel teknologi, tips & trik, serta update layanan dari WORKA SOLUTION".to_string(),
        url: format!("{}/news", config.server.base_url),
        og_image: format!("{}{}", config.server.base_url, settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string())),
    };

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_articles WHERE is_published = 1")
        .fetch_one(db)
        .await
        .unwrap_or(0);

    let news = sqlx::query_as::<_, NewsArticle>(
        "SELECT * FROM news_articles WHERE is_published = 1 ORDER BY published_at DESC LIMIT ? OFFSET ?"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(NewsView::from)
    .collect();

    let categories: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT category FROM news_articles WHERE is_published = 1 AND category IS NOT NULL"
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let total_pages = ((total as f64) / per_page as f64).ceil() as i32;

    let template = NewsTemplate {
        site,
        page,
        settings,
        news,
        categories,
        current_page: page_num,
        total_pages,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn news_detail_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;
    let slug = req.param::<String>("slug").unwrap_or_default();

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);

    let article = sqlx::query_as::<_, NewsArticle>(
        "SELECT * FROM news_articles WHERE slug = ? AND is_published = 1"
    )
    .bind(&slug)
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
    .map(NewsView::from);

    let article = match article {
        Some(a) => a,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Html(render_404(&site, &settings).await));
            return;
        }
    };

    let page = PageInfo {
        title: format!("{} - {}", article.title, config.app.brand_name),
        description: if article.excerpt.is_empty() { article.meta_description.clone() } else { article.excerpt.clone() },
        url: format!("{}/news/{}", config.server.base_url, slug),
        og_image: format!("{}{}", config.server.base_url, if article.thumbnail_path.is_empty() { settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string()) } else { article.thumbnail_path.clone() }),
    };

    let related_news = sqlx::query_as::<_, NewsArticle>(
        "SELECT * FROM news_articles WHERE is_published = 1 AND id != ? ORDER BY published_at DESC LIMIT 3"
    )
    .bind(&article.id)
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(NewsView::from)
    .collect();

    let template = NewsDetailTemplate {
        site,
        page,
        settings,
        article,
        related_news,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn about_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let db = &state.db;
    let config = &state.config;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);
    let page = PageInfo {
        title: format!("Tentang Kami - {}", config.app.brand_name),
        description: "WORKA SOLUTION - PT Perorangan dengan 14 KLBI teknologi. Menyediakan layanan IT terintegrasi sejak 2024.".to_string(),
        url: format!("{}/about", config.server.base_url),
        og_image: format!("{}{}", config.server.base_url, settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string())),
    };

    let team = vec![
        TeamMember {
            name: "Founder & CTO".to_string(),
            role: "Software Architect & Full Stack Developer".to_string(),
            bio: "Berpengalaman 10+ tahun dalam pengembangan software enterprise, arsitektur cloud, dan konsultasi transformasi digital.".to_string(),
            image: String::new(),
            social: site.social_media.clone(),
        },
    ];

    let klbi_codes = vec![
        KLBICode { code: "26210".to_string(), description: "Industri Komputer Dan/Atau Perakitan Komputer".to_string(), category: "Manufaktur".to_string() },
        KLBICode { code: "26220".to_string(), description: "Industri Perlengkapan Komputer".to_string(), category: "Manufaktur".to_string() },
        KLBICode { code: "26490".to_string(), description: "Industri Peralatan Audio dan Video Elektronik Lainnya".to_string(), category: "Manufaktur".to_string() },
        KLBICode { code: "46511".to_string(), description: "Perdagangan Besar Komputer dan Perlengkapan Komputer".to_string(), category: "Perdagangan".to_string() },
        KLBICode { code: "47411".to_string(), description: "Perdagangan Eceran Komputer Dan Perlengkapannya".to_string(), category: "Perdagangan".to_string() },
        KLBICode { code: "47415".to_string(), description: "Perdagangan Eceran Mesin Kantor".to_string(), category: "Perdagangan".to_string() },
        KLBICode { code: "47876".to_string(), description: "Perdagangan Eceran Kaki Lima Dan Los Pasar Mesin Kantor".to_string(), category: "Perdagangan".to_string() },
        KLBICode { code: "59122".to_string(), description: "Aktivitas Pascaproduksi Film, Video dan Program Televisi Oleh Swasta".to_string(), category: "Kreatif".to_string() },
        KLBICode { code: "62019".to_string(), description: "Aktivitas Pemrograman Komputer Lainnya".to_string(), category: "Layanan IT".to_string() },
        KLBICode { code: "62029".to_string(), description: "Aktivitas Konsultasi Komputer dan Manajemen Fasilitas Komputer Lainnya".to_string(), category: "Layanan IT".to_string() },
        KLBICode { code: "62090".to_string(), description: "Aktivitas Teknologi Informasi Dan Jasa Komputer Lainnya".to_string(), category: "Layanan IT".to_string() },
        KLBICode { code: "74130".to_string(), description: "Aktivitas Desain Komunikasi Visual/ Desain Grafis".to_string(), category: "Kreatif".to_string() },
        KLBICode { code: "95110".to_string(), description: "Reparasi Komputer dan Peralatan Sejenisnya".to_string(), category: "After-Sales".to_string() },
        KLBICode { code: "95120".to_string(), description: "Reparasi Peralatan Komunikasi".to_string(), category: "After-Sales".to_string() },
    ];

    let template = AboutTemplate {
        site,
        page,
        settings,
        team,
        klbi_codes,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn contact_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let config = &state.config;
    let db = &state.db;

    let settings = get_all_settings_map(db).await.unwrap_or_default();
    let site = build_site_info(config, &settings);
    let page = PageInfo {
        title: format!("Kontak - {}", config.app.brand_name),
        description: "Hubungi WORKA SOLUTION untuk kebutuhan teknologi Anda. Kami siap membantu proyek Anda.".to_string(),
        url: format!("{}/contact", config.server.base_url),
        og_image: format!("{}{}", config.server.base_url, settings.get("seo_og_image").cloned().unwrap_or_else(|| "/images/og-image.svg".to_string())),
    };

    let template = ContactTemplate {
        site,
        page,
        settings,
    };

    res.render(Text::Html(template.render().unwrap_or_default()));
}

#[handler]
pub async fn contact_submit_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let form = req.parse_form::<ContactForm>().await.unwrap_or_default();
    tracing::info!("Contact form submitted: {:?}", form);
    
    res.render(Json(crate::models::ApiResponse::ok(json!({
        "message": "Pesan berhasil dikirim. Kami akan menghubungi Anda segera."
    }))));
}

#[handler]
pub async fn newsletter_submit_handler(req: &mut Request, res: &mut Response) {
    #[derive(Deserialize, Default)]
    struct NewsletterForm {
        email: String,
    }

    let form = req.parse_form::<NewsletterForm>().await.unwrap_or_default();
    let message = if form.email.trim().is_empty() {
        "Masukkan alamat email yang valid."
    } else {
        "Terima kasih! Email Anda telah terdaftar."
    };

    res.render(Json(crate::models::ApiResponse::ok(json!({ "message": message }))));
}

#[derive(Debug, Deserialize, Default)]
pub struct ContactForm {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub subject: String,
    pub message: String,
}

// ============ Helpers ============

fn setting_value(settings: &HashMap<String, String>, key: &str, default: &str) -> String {
    settings
        .get(key)
        .map(|value| value.trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn build_site_info(config: &AppConfig, settings: &HashMap<String, String>) -> SiteInfo {
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

async fn render_404(site: &SiteInfo, _settings: &HashMap<String, String>) -> String {
    format!(r#"
<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 - Halaman Tidak Ditemukan - {}</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-slate-50 min-h-screen flex items-center justify-center">
    <div class="text-center px-4">
        <h1 class="text-9xl font-bold text-blue-600 mb-4">404</h1>
        <h2 class="text-2xl font-semibold text-slate-800 mb-4">Halaman Tidak Ditemukan</h2>
        <p class="text-slate-600 mb-8 max-w-md mx-auto">Maaf, halaman yang Anda cari tidak ada atau telah dipindahkan.</p>
        <a href="/" class="inline-block bg-blue-600 text-white px-6 py-3 rounded-lg hover:bg-blue-700 transition">Kembali ke Beranda</a>
    </div>
</body>
</html>
"#, site.brand_name)
}