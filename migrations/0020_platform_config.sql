-- Platform configuration: which ride-hailing platforms are active
-- Each platform has a name and maps to earnings columns in the trips table
CREATE TABLE IF NOT EXISTS platforms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,        -- e.g., 'Uber', 'Bolt', 'Careem'
    code TEXT NOT NULL UNIQUE,         -- e.g., 'uber', 'bolt', 'careem' (used as column key)
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed current platforms
INSERT INTO platforms (name, code) VALUES ('Uber', 'uber'), ('Bolt', 'bolt')
ON CONFLICT (code) DO NOTHING;

-- Platform-specific earnings per trip (replaces hardcoded uber_cash_aed/bolt_cash_aed)
CREATE TABLE IF NOT EXISTS trip_platform_earnings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id UUID NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    platform_id UUID NOT NULL REFERENCES platforms(id),
    amount_aed NUMERIC(12,2) NOT NULL DEFAULT 0,
    UNIQUE(trip_id, platform_id)
);

-- Migrate existing data from hardcoded columns to the new table
INSERT INTO trip_platform_earnings (trip_id, platform_id, amount_aed)
SELECT t.id, p.id, t.uber_cash_aed
FROM trips t, platforms p
WHERE p.code = 'uber' AND t.uber_cash_aed > 0;

INSERT INTO trip_platform_earnings (trip_id, platform_id, amount_aed)
SELECT t.id, p.id, t.bolt_cash_aed
FROM trips t, platforms p
WHERE p.code = 'bolt' AND t.bolt_cash_aed > 0;
