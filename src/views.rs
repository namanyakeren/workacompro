// Shared view models for Askama templates.
//
// Askama 0.12 cannot display `Option<T>` directly, so the database models are
// converted into these template-friendly structs where every field is a plain
// `String`/`bool`/number and dates are pre-formatted.

use crate::models::*;
use chrono::Datelike;
use serde::Serialize;

fn display_date(value: Option<chrono::DateTime<chrono::Utc>>, fallback: chrono::DateTime<chrono::Utc>, fmt: &str) -> String {
    value.unwrap_or(fallback).format(fmt).to_string()
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct HeroView {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub cta_text: String,
    pub cta_link: String,
    pub image_path: String,
    pub image_id: String,
    pub background_color: String,
    pub text_color: String,
    pub is_active: bool,
    pub sort_order: i32,
}

impl From<HeroBanner> for HeroView {
    fn from(h: HeroBanner) -> Self {
        let background_color = h
            .background_color
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "#f5c400".to_string());
        let text_color = h
            .text_color
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "#111111".to_string());
        Self {
            id: h.id.to_string(),
            title: h.title,
            subtitle: h.subtitle.unwrap_or_default(),
            description: h.description.unwrap_or_default(),
            cta_text: h.cta_text.unwrap_or_default(),
            cta_link: h.cta_link.unwrap_or_default(),
            image_path: h.image_path.unwrap_or_default(),
            image_id: String::new(),
            background_color,
            text_color,
            is_active: h.is_active,
            sort_order: h.sort_order,
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ServiceView {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub short_description: String,
    pub full_description: String,
    pub icon: String,
    pub image_path: String,
    pub image_id: String,
    pub category: String,
    pub features: String,
    pub is_active: bool,
    pub is_featured: bool,
    pub sort_order: i32,
}

impl From<Service> for ServiceView {
    fn from(s: Service) -> Self {
        let features = s
            .features
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "[]".to_string());
        Self {
            id: s.id.to_string(),
            title: s.title,
            slug: s.slug,
            short_description: s.short_description.unwrap_or_default(),
            full_description: s.full_description.unwrap_or_default(),
            icon: s.icon.unwrap_or_default(),
            image_path: s.image_path.unwrap_or_default(),
            image_id: String::new(),
            category: s.category.unwrap_or_default(),
            features,
            is_active: s.is_active,
            is_featured: s.is_featured,
            sort_order: s.sort_order,
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct PortfolioView {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub short_description: String,
    pub full_description: String,
    pub client_name: String,
    pub project_url: String,
    pub thumbnail_path: String,
    pub thumbnail_id: String,
    pub gallery_images: String,
    pub technologies: String,
    pub category: String,
    pub start_date: String,
    pub end_date: String,
    pub start_date_display: String,
    pub end_date_display: String,
    pub is_active: bool,
    pub is_featured: bool,
    pub sort_order: i32,
}

impl From<Portfolio> for PortfolioView {
    fn from(p: Portfolio) -> Self {
        let gallery_images = p
            .gallery_images
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "[]".to_string());
        let technologies = p
            .technologies
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "[]".to_string());
        let start_date_display = p
            .start_date
            .map(|d| d.format("%b %Y").to_string())
            .unwrap_or_default();
        let end_date_display = p
            .end_date
            .map(|d| d.format("%b %Y").to_string())
            .unwrap_or_default();
        Self {
            id: p.id.to_string(),
            title: p.title,
            slug: p.slug,
            short_description: p.short_description.unwrap_or_default(),
            full_description: p.full_description.unwrap_or_default(),
            client_name: p.client_name.unwrap_or_default(),
            project_url: p.project_url.unwrap_or_default(),
            thumbnail_path: p.thumbnail_path.unwrap_or_default(),
            thumbnail_id: String::new(),
            gallery_images,
            technologies,
            category: p.category.unwrap_or_default(),
            start_date: p.start_date.map(|d| d.to_string()).unwrap_or_default(),
            end_date: p.end_date.map(|d| d.to_string()).unwrap_or_default(),
            start_date_display,
            end_date_display,
            is_active: p.is_active,
            is_featured: p.is_featured,
            sort_order: p.sort_order,
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct NewsView {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub thumbnail_path: String,
    pub thumbnail_id: String,
    pub author: String,
    pub category: String,
    pub tags: String,
    pub is_published: bool,
    pub published_at: String,
    pub published_iso: String,
    pub published_display: String,
    pub published_display_long: String,
    pub meta_title: String,
    pub meta_description: String,
}

impl From<NewsArticle> for NewsView {
    fn from(n: NewsArticle) -> Self {
        let tags = n
            .tags
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "[]".to_string());
        let published_iso = display_date(n.published_at, n.created_at, "%Y-%m-%d");
        let published_display = display_date(n.published_at, n.created_at, "%d %b %Y");
        let published_display_long = display_date(n.published_at, n.created_at, "%d %B %Y");
        Self {
            id: n.id.to_string(),
            title: n.title,
            slug: n.slug,
            excerpt: n.excerpt.unwrap_or_default(),
            content: n.content,
            thumbnail_path: n.thumbnail_path.unwrap_or_default(),
            thumbnail_id: String::new(),
            author: n.author.unwrap_or_default(),
            category: n.category.unwrap_or_default(),
            tags,
            is_published: n.is_published,
            published_at: n.published_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            published_iso,
            published_display,
            published_display_long,
            meta_title: n.meta_title.unwrap_or_default(),
            meta_description: n.meta_description.unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct MediaView {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: i32,
    pub height: i32,
    pub path: String,
    pub alt_text: String,
    pub uploaded_by: String,
}

impl From<MediaFile> for MediaView {
    fn from(m: MediaFile) -> Self {
        Self {
            id: m.id.to_string(),
            filename: m.filename,
            original_name: m.original_name,
            mime_type: m.mime_type,
            size_bytes: m.size_bytes,
            width: m.width.unwrap_or(0),
            height: m.height.unwrap_or(0),
            path: m.path,
            alt_text: m.alt_text.unwrap_or_default(),
            uploaded_by: m.uploaded_by.unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct SettingView {
    pub id: String,
    pub key: String,
    pub value: String,
    pub description: String,
    pub group: String,
    pub display_label: String,
    pub is_color: bool,
    pub is_textarea: bool,
    pub is_required: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct SettingGroupView {
    pub key: String,
    pub name: String,
    pub icon: String,
    pub settings: Vec<SettingView>,
}

fn title_case_key(key: &str) -> String {
    key.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl From<Setting> for SettingView {
    fn from(s: Setting) -> Self {
        let description = s.description.unwrap_or_default();
        let display_label = if description.trim().is_empty() {
            title_case_key(&s.key)
        } else {
            description.clone()
        };
        let is_color = s.key.ends_with("_color");
        let is_textarea = matches!(
            s.key.as_str(),
            "site_description" | "seo_meta_description" | "seo_meta_keywords"
        );
        let is_required = matches!(s.key.as_str(), "site_brand_name" | "site_email" | "site_phone");
        Self {
            id: s.id.to_string(),
            key: s.key,
            value: s.value,
            description,
            group: s.group,
            display_label,
            is_color,
            is_textarea,
            is_required,
        }
    }
}

/// Admin templates use the same shapes as the public views.
pub type AdminHeroView = HeroView;
pub type AdminServiceView = ServiceView;
pub type AdminPortfolioView = PortfolioView;
pub type AdminNewsView = NewsView;
pub type AdminMediaView = MediaView;
pub type AdminSettingView = SettingView;
pub type AdminSettingGroupView = SettingGroupView;

// `Datelike` is used above via chrono's `format`; keep the import referenced.
#[allow(dead_code)]
fn _keep_datelike(d: chrono::NaiveDate) -> i32 {
    d.year()
}
