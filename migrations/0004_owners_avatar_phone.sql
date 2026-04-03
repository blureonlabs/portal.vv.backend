-- Sprint 11: Owners, avatar, phone, direct driver creation

-- Add owner role to enum
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'owner';

-- Add phone + avatar to profiles
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS phone TEXT;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS avatar_url TEXT;

-- Owners table
CREATE TABLE IF NOT EXISTS owners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL UNIQUE REFERENCES profiles(id),
    company_name TEXT,
    notes TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Link vehicles to owners (null = company-owned)
ALTER TABLE vehicles ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES owners(id);
