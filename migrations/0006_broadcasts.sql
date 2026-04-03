-- Sprint 11: Broadcast / Communications
DO $$ BEGIN CREATE TYPE broadcast_channel AS ENUM ('email', 'whatsapp'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE broadcast_target AS ENUM ('all_drivers', 'selected_drivers'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE broadcast_status AS ENUM ('draft', 'sending', 'sent', 'failed'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE IF NOT EXISTS broadcasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    channel broadcast_channel NOT NULL DEFAULT 'email',
    target broadcast_target NOT NULL DEFAULT 'all_drivers',
    target_driver_ids UUID[],
    sent_by UUID NOT NULL REFERENCES profiles(id),
    recipient_count INT NOT NULL DEFAULT 0,
    status broadcast_status NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_broadcasts_created ON broadcasts(created_at DESC);
