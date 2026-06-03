INSERT INTO settings (key, value) VALUES ('salary_auto_generate_day', '25')
ON CONFLICT (key) DO NOTHING;

INSERT INTO settings (key, value) VALUES ('salary_auto_generate_enabled', 'false')
ON CONFLICT (key) DO NOTHING;
