-- Add commission_rate to settings (default 75% — driver keeps 75% of trip revenue)
INSERT INTO settings (key, value) VALUES ('commission_rate', '0.75')
ON CONFLICT (key) DO NOTHING;
