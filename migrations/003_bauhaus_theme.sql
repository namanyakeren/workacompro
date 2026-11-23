-- Theme refresh: Bauhaus palette.
-- Vermilion primary, cobalt secondary, chrome-yellow hero canvas.
-- Existing installations pick these up automatically on the next start.

UPDATE settings SET value = '"#e03c31"', description = 'Warna primer (vermilion)'
WHERE key = 'primary_color';

UPDATE settings SET value = '"#0057b8"', description = 'Warna sekunder (cobalt)'
WHERE key = 'secondary_color';

UPDATE settings SET value = '"#f5c400"', description = 'Warna background hero (kuning)'
WHERE key = 'hero_background_color';

UPDATE settings SET value = '"#111111"', description = 'Warna teks hero (hitam)'
WHERE key = 'hero_text_color';
