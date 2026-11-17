use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============ Core Models ============

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct HeroBanner {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cta_text: Option<String>,
    pub cta_link: Option<String>,
    pub image_path: Option<String>,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub icon: Option<String>, // Lucide icon name or custom SVG
    pub image_path: Option<String>,
    pub category: Option<String>,
    pub features: Option<String>, // JSON array of features
    pub is_active: bool,
    pub is_featured: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub client_name: Option<String>,
    pub project_url: Option<String>,
    pub thumbnail_path: Option<String>,
    pub gallery_images: Option<String>, // JSON array
    pub technologies: Option<String>, // JSON array
    pub category: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub is_featured: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub content: String,
    pub thumbnail_path: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>, // JSON array
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Setting {
    pub id: String,
    pub key: String,
    pub value: String, // JSON string for complex values
    pub description: Option<String>,
    pub group: String, // "general", "seo", "contact", "social", "appearance"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub path: String,
    pub alt_text: Option<String>,
    pub uploaded_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============ DTOs for Forms ============

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HeroBannerForm {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cta_text: Option<String>,
    pub cta_link: Option<String>,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
    pub image_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ServiceForm {
    pub title: String,
    pub slug: String,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub features: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub image_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PortfolioForm {
    pub title: String,
    pub slug: String,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub client_name: Option<String>,
    pub project_url: Option<String>,
    pub category: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub sort_order: Option<i32>,
    pub thumbnail_id: Option<String>,
    pub gallery_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NewsForm {
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_published: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub thumbnail_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

// ============ Response Types ============

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), message: None, errors: None }
    }
    pub fn err(message: String, errors: Option<serde_json::Value>) -> Self {
        Self { success: false, data: None, message: Some(message), errors }
    }
}

// ============ Template Context Types ============

#[derive(Debug, Serialize, Clone)]
pub struct SiteInfo {
    pub brand_name: String,
    pub legal_name: String,
    pub tagline: String,
    pub description: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub logo: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub hero_background_color: String,
    pub hero_text_color: String,
    pub social_media: SocialMediaInfo,
    pub seo: SeoInfo,
    pub current_year: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct SocialMediaInfo {
    pub instagram: String,
    pub linkedin: String,
    pub twitter: String,
    pub youtube: String,
    pub github: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SeoInfo {
    pub meta_title: String,
    pub meta_description: String,
    pub meta_keywords: String,
    pub og_image: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PageInfo {
    pub title: String,
    pub description: String,
    pub url: String,
    pub og_image: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AdminPageInfo {
    pub title: String,
    pub breadcrumb: Vec<BreadcrumbItem>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DashboardStats {
    pub total_services: i64,
    pub total_portfolios: i64,
    pub total_news: i64,
    pub published_news: i64,
    pub total_media: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    pub bio: String,
    pub image: String,
    pub social: SocialMediaInfo,
}

#[derive(Debug, Serialize, Clone)]
pub struct KLBICode {
    pub code: String,
    pub description: String,
    pub category: String,
}