INSERT INTO settings (key, value) VALUES ('cash_shortfall_threshold_aed', '500') ON CONFLICT (key) DO NOTHING;
INSERT INTO settings (key, value) VALUES ('auto_salary_enabled', 'false') ON CONFLICT (key) DO NOTHING;
INSERT INTO settings (key, value) VALUES ('pay_cycle_day', '25') ON CONFLICT (key) DO NOTHING;
