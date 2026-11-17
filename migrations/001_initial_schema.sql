-- Migration: Initial schema
CREATE TABLE IF NOT EXISTS hero_banners (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    subtitle TEXT,
    description TEXT,
    cta_text TEXT,
    cta_link TEXT,
    image_path TEXT,
    background_color TEXT,
    text_color TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    short_description TEXT,
    full_description TEXT,
    icon TEXT,
    image_path TEXT,
    category TEXT,
    features TEXT, -- JSON array
    is_active INTEGER NOT NULL DEFAULT 1,
    is_featured INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_services_slug ON services(slug);
CREATE INDEX IF NOT EXISTS idx_services_active ON services(is_active);

CREATE TABLE IF NOT EXISTS portfolios (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    short_description TEXT,
    full_description TEXT,
    client_name TEXT,
    project_url TEXT,
    thumbnail_path TEXT,
    gallery_images TEXT, -- JSON array
    technologies TEXT, -- JSON array
    category TEXT,
    start_date TEXT,
    end_date TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_featured INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_portfolios_slug ON portfolios(slug);
CREATE INDEX IF NOT EXISTS idx_portfolios_active ON portfolios(is_active);

CREATE TABLE IF NOT EXISTS news_articles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    excerpt TEXT,
    content TEXT NOT NULL,
    thumbnail_path TEXT,
    author TEXT,
    category TEXT,
    tags TEXT, -- JSON array
    is_published INTEGER NOT NULL DEFAULT 0,
    published_at TEXT,
    meta_title TEXT,
    meta_description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_news_slug ON news_articles(slug);
CREATE INDEX IF NOT EXISTS idx_news_published ON news_articles(is_published, published_at);

CREATE TABLE IF NOT EXISTS settings (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL,
    description TEXT,
    "group" TEXT NOT NULL DEFAULT 'general',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_settings_group ON settings("group");

CREATE TABLE IF NOT EXISTS media_files (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    original_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    path TEXT NOT NULL,
    alt_text TEXT,
    uploaded_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_media_mime ON media_files(mime_type);

CREATE TABLE IF NOT EXISTS admin_users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    email TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    last_login TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default admin user (password: worka2024!)
INSERT OR IGNORE INTO admin_users (id, username, password_hash, full_name, email, is_active, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'worka_admin',
    '$argon2id$v=19$m=19456,t=2,p=1$CKJ6R0SQzwvpHD74KtU8Vw$yVU4Ym5LX6jPNPsAhGuld98HPiohvWmAZOvSqNrKPco',
    'WORKA Admin',
    'admin@worka.solution',
    1,
    datetime('now'),
    datetime('now')
);

-- Default settings
INSERT OR IGNORE INTO settings (id, key, value, description, "group") VALUES
('00000000-0000-0000-0000-000000000010', 'site_brand_name', '"WORKA SOLUTION"', 'Nama brand yang ditampilkan', 'general'),
('00000000-0000-0000-0000-000000000011', 'site_legal_name', '"PT. WIH ORANG KAYA"', 'Nama hukum perusahaan (hanya footer)', 'general'),
('00000000-0000-0000-0000-000000000012', 'site_tagline', '"Solusi Teknologi Terintegrasi untuk Bisnis Modern"', 'Tagline utama', 'general'),
('00000000-0000-0000-0000-000000000013', 'site_description', '"WORKA SOLUTION menyediakan layanan teknologi informasi lengkap..."', 'Deskripsi singkat', 'general'),
('00000000-0000-0000-0000-000000000014', 'site_address', '"Jl. Teknologi No. 123, Jakarta Selatan 12345"', 'Alamat lengkap', 'contact'),
('00000000-0000-0000-0000-000000000015', 'site_phone', '"+62 21 1234 5678"', 'Nomor telepon', 'contact'),
('00000000-0000-0000-0000-000000000016', 'site_email', '"hello@worka.solution"', 'Email kontak', 'contact'),
('00000000-0000-0000-0000-000000000017', 'site_website', '"https://worka.solution"', 'Website resmi', 'contact'),
('00000000-0000-0000-0000-000000000018', 'social_instagram', '"https://instagram.com/workasolution"', 'Instagram URL', 'social'),
('00000000-0000-0000-0000-000000000019', 'social_linkedin', '"https://linkedin.com/company/worka-solution"', 'LinkedIn URL', 'social'),
('00000000-0000-0000-0000-000000000020', 'social_twitter', '"https://twitter.com/workasolution"', 'Twitter/X URL', 'social'),
('00000000-0000-0000-0000-000000000021', 'social_youtube', '"https://youtube.com/@workasolution"', 'YouTube URL', 'social'),
('00000000-0000-0000-0000-000000000022', 'social_github', '"https://github.com/worka-solution"', 'GitHub URL', 'social'),
('00000000-0000-0000-0000-000000000030', 'seo_meta_title', '"WORKA SOLUTION - Solusi Teknologi Terintegrasi"', 'Meta title default', 'seo'),
('00000000-0000-0000-0000-000000000031', 'seo_meta_description', '"WORKA SOLUTION menyediakan layanan IT lengkap..."', 'Meta description default', 'seo'),
('00000000-0000-0000-0000-000000000032', 'seo_meta_keywords', '"software development, IT consulting, graphic design, network security, hardware repair"', 'Meta keywords', 'seo'),
('00000000-0000-0000-0000-000000000033', 'seo_og_image', '"/images/og-image.svg"', 'Open Graph image default', 'seo'),
('00000000-0000-0000-0000-000000000040', 'hero_background_color', '"#0f172a"', 'Warna background hero', 'appearance'),
('00000000-0000-0000-0000-000000000041', 'hero_text_color', '"#ffffff"', 'Warna teks hero', 'appearance'),
('00000000-0000-0000-0000-000000000042', 'primary_color', '"#3b82f6"', 'Warna primer (blue-500)', 'appearance'),
('00000000-0000-0000-0000-000000000043', 'secondary_color', '"#64748b"', 'Warna sekunder (slate-500)', 'appearance');