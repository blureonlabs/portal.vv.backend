-- Fix default commission rate: spec says 30% (0.30), was incorrectly set to 75% (0.75)
UPDATE settings SET value = '0.30' WHERE key = 'commission_rate' AND value = '0.75';

-- Ensure fixed car charging high setting exists
INSERT INTO settings (key, value) VALUES ('salary_fixed_car_high_aed', '1600')
ON CONFLICT (key) DO NOTHING;
