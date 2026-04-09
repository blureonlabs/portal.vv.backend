ALTER TABLE drivers ADD COLUMN IF NOT EXISTS room_rent_aed NUMERIC(12,2) NOT NULL DEFAULT 0;
ALTER TABLE drivers ADD COLUMN IF NOT EXISTS commission_rate NUMERIC(5,4);
-- commission_rate is nullable: NULL means use global default from settings
