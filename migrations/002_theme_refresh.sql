-- Theme refresh: warm editorial palette.
-- Deep pine as primary, terracotta as secondary, ivory hero canvas.
-- Existing installations pick these up automatically on the next start.

UPDATE settings SET value = '"#12463d"', description = 'Warna primer (pine)'
WHERE key = 'primary_color';

UPDATE settings SET value = '"#e2542c"', description = 'Warna sekunder (terracotta)'
WHERE key = 'secondary_color';

UPDATE settings SET value = '"#f6f1e8"', description = 'Warna background hero'
WHERE key = 'hero_background_color';

UPDATE settings SET value = '"#151310"', description = 'Warna teks hero'
WHERE key = 'hero_text_color';
