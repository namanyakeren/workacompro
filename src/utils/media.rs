use anyhow::Result;
use image::{ImageFormat, DynamicImage};
use std::io::Cursor;
use uuid::Uuid;
use chrono::Utc;
use crate::config::AppConfig;
use crate::models::MediaFile;

pub fn generate_slug(title: &str) -> String {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    
    if slug.is_empty() {
        format!("item-{}", Uuid::new_v4().simple())
    } else {
        slug
    }
}

pub fn ensure_unique_slug(base_slug: &str, existing: &[String]) -> String {
    if !existing.contains(&base_slug.to_string()) {
        return base_slug.to_string();
    }
    
    for i in 2..1000 {
        let candidate = format!("{}-{}", base_slug, i);
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    
    format!("{}-{}", base_slug, Uuid::new_v4().simple())
}

pub fn validate_file_extension(filename: &str, allowed: &[String]) -> bool {
    let ext = filename
        .split('.')
        .last()
        .unwrap_or("")
        .to_lowercase();
    allowed.iter().any(|a| a.to_lowercase() == ext)
}

pub fn get_mime_type(filename: &str) -> &'static str {
    let ext = filename.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

pub async fn process_upload(
    file_data: Vec<u8>,
    original_name: String,
    config: &AppConfig,
    uploaded_by: Option<String>,
) -> Result<MediaFile> {
    // Validate size
    let max_bytes = config.upload.max_size_mb * 1024 * 1024;
    if file_data.len() as u64 > max_bytes {
        anyhow::bail!("File size exceeds maximum allowed ({} MB)", config.upload.max_size_mb);
    }

    // Validate extension
    if !validate_file_extension(&original_name, &config.upload.allowed_extensions) {
        anyhow::bail!("File type not allowed");
    }

    // Generate unique filename
    let ext = original_name.split('.').last().unwrap_or("bin").to_lowercase();
    let filename = format!("{}.{}", Uuid::new_v4().simple(), ext);
    
    // Create upload directory
    let upload_dir = &config.upload.upload_dir;
    tokio::fs::create_dir_all(upload_dir).await?;

    // Process image if it's an image
    let (width, height, processed_data) = if get_mime_type(&original_name).starts_with("image/") && ext != "svg" {
        let img = image::load_from_memory(&file_data)?;
        let (w, h) = (img.width() as i32, img.height() as i32);
        
        // Resize if too large (max 1920px width)
        let processed = if w > 1920 {
            let ratio = 1920.0 / w as f32;
            let new_h = (h as f32 * ratio) as u32;
            img.resize(1920, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };
        
        // Save as WebP for better compression (except SVG)
        let mut buf = Vec::new();
        let format = if ext == "png" { ImageFormat::Png } else { ImageFormat::WebP };
        processed.write_to(&mut Cursor::new(&mut buf), format)?;
        (Some(w), Some(h), buf)
    } else {
        (None, None, file_data)
    };

    let final_filename = if ext != "svg" && get_mime_type(&original_name).starts_with("image/") {
        format!("{}.webp", Uuid::new_v4().simple())
    } else {
        filename
    };

    let file_path = upload_dir.join(&final_filename);
    tokio::fs::write(&file_path, &processed_data).await?;

    let mime_type = if final_filename.ends_with(".webp") { "image/webp" } else { get_mime_type(&original_name) };

    Ok(MediaFile {
        id: Uuid::new_v4().to_string(),
        filename: final_filename.clone(),
        original_name,
        mime_type: mime_type.to_string(),
        size_bytes: processed_data.len() as i64,
        width,
        height,
        path: format!("{}/{}", config.upload.serve_path, final_filename),
        alt_text: None,
        uploaded_by,
        created_at: Utc::now(),
    })
}

pub async fn delete_file(file_path: &str, config: &AppConfig) -> Result<()> {
    let full_path = config.upload.upload_dir.join(file_path.strip_prefix(&format!("{}/", config.upload.serve_path)).unwrap_or(file_path));
    if full_path.exists() {
        tokio::fs::remove_file(full_path).await?;
    }
    Ok(())
}