-- Rename other_aed to uber_cash_aed and add bolt_cash_aed
ALTER TABLE trips ADD COLUMN IF NOT EXISTS uber_cash_aed NUMERIC(12,2) NOT NULL DEFAULT 0;
ALTER TABLE trips ADD COLUMN IF NOT EXISTS bolt_cash_aed NUMERIC(12,2) NOT NULL DEFAULT 0;

-- Migrate existing other_aed data to uber_cash_aed
UPDATE trips SET uber_cash_aed = other_aed WHERE other_aed > 0;
