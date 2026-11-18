# WORKA SOLUTION

Website company profile untuk **WORKA SOLUTION** (PT. WIH ORANG KAYA) dengan CMS admin tersembunyi.

Stack: Rust (Salvo) · SQLite · Askama · HTMX · Tailwind CSS (CDN).

## Menjalankan lokal

```bash
cargo run --release
```

Server berjalan di `http://localhost:8080` (ubah dengan `PORT=...`).

## Docker

```bash
cp .env.example .env      # opsional, semua nilai punya default
docker compose up -d --build
```

Port publik diatur lewat `PORT` di `.env` (default `8080`).
Database dan file upload disimpan di volume `worka_data` dan `worka_uploads`.

## Akses admin

Halaman admin **tidak punya tombol di UI**. Tiga cara masuk:

| Perangkat | Cara |
| --- | --- |
| Desktop | Ketik `admin` (huruf biasa) di halaman mana pun |
| Desktop | Tekan `Ctrl`/`Cmd` + `Shift` + `A` |
| Mobile | Tahan (long-press) teks nama legal di footer selama 1,2 detik |

Atau buka langsung `/admin/login`.

Login default (di-seed oleh migrasi database):

- Username: `worka_admin`
- Password: `worka2024!`

Ganti password sebelum produksi:

```bash
cargo run --example hash_password -- 'password-baru'
docker compose exec worka-solution sqlite3 /app/data/worka.db \
  "UPDATE admin_users SET password_hash='<hash-baru>' WHERE username='worka_admin';"
```

## Tema

Palet dan tipografi publik:

- Latar: ivory `#f6f1e8`
- Primer: pine `#12463d`
- Sekunder: terracotta `#e2542c`
- Display: Fraunces, body: Plus Jakarta Sans

Warna primer/sekunder dapat diubah dari **Admin → Pengaturan → Tampilan**.
