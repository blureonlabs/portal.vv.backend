-- Sprint 3: allow admin to enable/disable drivers entering their own trip earnings
ALTER TABLE drivers ADD COLUMN self_entry_enabled BOOLEAN NOT NULL DEFAULT false;
