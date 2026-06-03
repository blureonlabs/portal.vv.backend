-- Configurable expense categories (parallel to expense_category ENUM)
CREATE TABLE IF NOT EXISTS config_expense_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    code TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO config_expense_categories (name, code, sort_order) VALUES
    ('Fuel', 'fuel', 1), ('Maintenance', 'maintenance', 2),
    ('Toll', 'toll', 3), ('Insurance', 'insurance', 4),
    ('Fines', 'fines', 5), ('Other', 'other', 99)
ON CONFLICT (code) DO NOTHING;

-- Configurable leave types (parallel to leave_type ENUM)
CREATE TABLE IF NOT EXISTS config_leave_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    code TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO config_leave_types (name, code, sort_order) VALUES
    ('Leave', 'leave', 1), ('Permission', 'permission', 2)
ON CONFLICT (code) DO NOTHING;

-- Configurable document types (parallel to document_type ENUM)
CREATE TABLE IF NOT EXISTS config_document_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    code TEXT NOT NULL UNIQUE,
    applies_to TEXT NOT NULL DEFAULT 'both', -- 'driver', 'vehicle', 'both'
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO config_document_types (name, code, applies_to, sort_order) VALUES
    ('Driving License', 'license', 'driver', 1),
    ('Visa', 'visa', 'driver', 2),
    ('Passport', 'passport', 'driver', 3),
    ('Emirates ID', 'emirates_id', 'driver', 4),
    ('Medical Certificate', 'medical', 'driver', 5),
    ('Registration Card', 'registration_card', 'vehicle', 6),
    ('Insurance Certificate', 'insurance_certificate', 'vehicle', 7),
    ('Receipt', 'receipt', 'both', 8),
    ('Other', 'other', 'both', 99)
ON CONFLICT (code) DO NOTHING;

-- Add sort_order to existing platforms table if not exists
ALTER TABLE platforms ADD COLUMN IF NOT EXISTS sort_order INT NOT NULL DEFAULT 0;
UPDATE platforms SET sort_order = 1 WHERE code = 'uber' AND sort_order = 0;
UPDATE platforms SET sort_order = 2 WHERE code = 'bolt' AND sort_order = 0;
