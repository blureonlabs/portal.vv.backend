-- FMS Initial Schema
-- All monetary values: NUMERIC(12,2) AED
-- All timestamps: TIMESTAMPTZ stored UTC, displayed Asia/Dubai (UTC+4)
-- RLS enabled on every table
-- created_at / updated_at auto-managed by trigger

-- ─── Enums ───────────────────────────────────────────────────────────────────

CREATE TYPE user_role AS ENUM ('super_admin', 'accountant', 'hr', 'driver');
CREATE TYPE salary_type AS ENUM ('commission', 'target_high', 'target_low');
CREATE TYPE invite_status AS ENUM ('pending', 'accepted', 'revoked', 'expired');
CREATE TYPE advance_status AS ENUM ('pending', 'approved', 'rejected', 'paid');
CREATE TYPE leave_type AS ENUM ('leave', 'permission');
CREATE TYPE leave_status AS ENUM ('pending', 'approved', 'rejected');
CREATE TYPE trip_source AS ENUM ('manual', 'csv_import', 'uber_api');
CREATE TYPE expense_category AS ENUM ('fuel', 'maintenance', 'toll', 'insurance', 'fines', 'other');
CREATE TYPE payment_method AS ENUM ('cash', 'bank_transfer');
CREATE TYPE vehicle_status AS ENUM ('available', 'assigned', 'inactive');

-- ─── Auto-update trigger ──────────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ─── profiles ─────────────────────────────────────────────────────────────────

CREATE TABLE profiles (
  id           UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  role         user_role NOT NULL,
  full_name    TEXT NOT NULL,
  email        TEXT NOT NULL UNIQUE,
  is_active    BOOLEAN NOT NULL DEFAULT true,
  invited_by   UUID REFERENCES profiles(id),
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER profiles_updated_at BEFORE UPDATE ON profiles
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE profiles ENABLE ROW LEVEL SECURITY;

-- ─── invites ──────────────────────────────────────────────────────────────────

CREATE TABLE invites (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email        TEXT NOT NULL,
  role         user_role NOT NULL,
  token_hash   TEXT NOT NULL UNIQUE,
  invited_by   UUID NOT NULL REFERENCES profiles(id),
  status       invite_status NOT NULL DEFAULT 'pending',
  expires_at   TIMESTAMPTZ NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER invites_updated_at BEFORE UPDATE ON invites
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE invites ENABLE ROW LEVEL SECURITY;

-- ─── drivers ─────────────────────────────────────────────────────────────────

CREATE TABLE drivers (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_id   UUID NOT NULL UNIQUE REFERENCES profiles(id),
  nationality  TEXT NOT NULL,
  salary_type  salary_type NOT NULL,
  is_active    BOOLEAN NOT NULL DEFAULT true,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER drivers_updated_at BEFORE UPDATE ON drivers
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE drivers ENABLE ROW LEVEL SECURITY;

-- ─── driver_edits (immutable) ─────────────────────────────────────────────────

CREATE TABLE driver_edits (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id    UUID NOT NULL REFERENCES drivers(id),
  changed_by   UUID NOT NULL REFERENCES profiles(id),
  field        TEXT NOT NULL,
  old_val      TEXT,
  new_val      TEXT,
  changed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE driver_edits ENABLE ROW LEVEL SECURITY;

-- ─── vehicles ─────────────────────────────────────────────────────────────────

CREATE TABLE vehicles (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  plate_number        TEXT NOT NULL UNIQUE,
  make                TEXT NOT NULL,
  model               TEXT NOT NULL,
  year                INT NOT NULL,
  color               TEXT,
  registration_date   DATE,
  registration_expiry DATE,
  insurance_expiry    DATE,
  status              vehicle_status NOT NULL DEFAULT 'available',
  is_active           BOOLEAN NOT NULL DEFAULT true,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER vehicles_updated_at BEFORE UPDATE ON vehicles
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE vehicles ENABLE ROW LEVEL SECURITY;

-- ─── vehicle_assignments ──────────────────────────────────────────────────────

CREATE TABLE vehicle_assignments (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  vehicle_id     UUID NOT NULL REFERENCES vehicles(id),
  driver_id      UUID NOT NULL REFERENCES drivers(id),
  assigned_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  unassigned_at  TIMESTAMPTZ,
  assigned_by    UUID NOT NULL REFERENCES profiles(id)
);

ALTER TABLE vehicle_assignments ENABLE ROW LEVEL SECURITY;

-- ─── vehicle_service (immutable) ─────────────────────────────────────────────

CREATE TABLE vehicle_service (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  vehicle_id   UUID NOT NULL REFERENCES vehicles(id),
  service_date DATE NOT NULL,
  type         TEXT NOT NULL,
  description  TEXT,
  cost         NUMERIC(12,2) NOT NULL DEFAULT 0,
  next_due     DATE,
  logged_by    UUID NOT NULL REFERENCES profiles(id),
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE vehicle_service ENABLE ROW LEVEL SECURITY;

-- ─── trips ───────────────────────────────────────────────────────────────────

CREATE TABLE trips (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id    UUID NOT NULL REFERENCES drivers(id),
  vehicle_id   UUID REFERENCES vehicles(id),
  entered_by   UUID NOT NULL REFERENCES profiles(id),
  trip_date    DATE NOT NULL,
  cash_aed     NUMERIC(12,2) NOT NULL DEFAULT 0,
  card_aed     NUMERIC(12,2) NOT NULL DEFAULT 0,
  other_aed    NUMERIC(12,2) NOT NULL DEFAULT 0,
  source       trip_source NOT NULL DEFAULT 'manual',
  notes        TEXT,
  is_deleted   BOOLEAN NOT NULL DEFAULT false,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trips_updated_at BEFORE UPDATE ON trips
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE trips ENABLE ROW LEVEL SECURITY;

-- ─── expenses ────────────────────────────────────────────────────────────────

CREATE TABLE expenses (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id    UUID REFERENCES drivers(id),
  entered_by   UUID NOT NULL REFERENCES profiles(id),
  amount_aed   NUMERIC(12,2) NOT NULL,
  category     expense_category NOT NULL,
  date         DATE NOT NULL,
  receipt_url  TEXT,
  notes        TEXT,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER expenses_updated_at BEFORE UPDATE ON expenses
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE expenses ENABLE ROW LEVEL SECURITY;

-- ─── cash_handovers ──────────────────────────────────────────────────────────

CREATE TABLE cash_handovers (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id    UUID NOT NULL REFERENCES drivers(id),
  amount_aed   NUMERIC(12,2) NOT NULL,
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  verified_by  UUID NOT NULL REFERENCES profiles(id)
);

ALTER TABLE cash_handovers ENABLE ROW LEVEL SECURITY;

-- ─── advances ────────────────────────────────────────────────────────────────

CREATE TABLE advances (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id         UUID NOT NULL REFERENCES drivers(id),
  amount_aed        NUMERIC(12,2) NOT NULL,
  reason            TEXT NOT NULL,
  status            advance_status NOT NULL DEFAULT 'pending',
  rejection_reason  TEXT,
  payment_date      DATE,
  method            payment_method,
  carry_forward_aed NUMERIC(12,2) NOT NULL DEFAULT 0,
  salary_period     DATE,
  actioned_by       UUID REFERENCES profiles(id),
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER advances_updated_at BEFORE UPDATE ON advances
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE advances ENABLE ROW LEVEL SECURITY;

-- ─── leave_requests ──────────────────────────────────────────────────────────

CREATE TABLE leave_requests (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id        UUID NOT NULL REFERENCES drivers(id),
  type             leave_type NOT NULL,
  from_date        DATE NOT NULL,
  to_date          DATE NOT NULL,
  reason           TEXT NOT NULL,
  status           leave_status NOT NULL DEFAULT 'pending',
  actioned_by      UUID REFERENCES profiles(id),
  rejection_reason TEXT,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER leave_requests_updated_at BEFORE UPDATE ON leave_requests
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE leave_requests ENABLE ROW LEVEL SECURITY;

-- ─── salaries ────────────────────────────────────────────────────────────────

CREATE TABLE salaries (
  id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id                 UUID NOT NULL REFERENCES drivers(id),
  period_month              DATE NOT NULL,
  salary_type_snapshot      salary_type NOT NULL,
  total_earnings_aed        NUMERIC(12,2) NOT NULL,
  car_charging_aed          NUMERIC(12,2) NOT NULL DEFAULT 0,
  car_charging_used_aed     NUMERIC(12,2),
  salik_used_aed            NUMERIC(12,2) NOT NULL DEFAULT 0,
  salik_refund_aed          NUMERIC(12,2) NOT NULL DEFAULT 0,
  salik_aed                 NUMERIC(12,2) NOT NULL DEFAULT 0,
  rta_fine_aed              NUMERIC(12,2) NOT NULL DEFAULT 0,
  card_service_charges_aed  NUMERIC(12,2) NOT NULL DEFAULT 0,
  total_cash_received_aed   NUMERIC(12,2) NOT NULL DEFAULT 0,
  cash_not_handover_aed     NUMERIC(12,2) NOT NULL DEFAULT 0,
  room_rent_aed             NUMERIC(12,2),
  target_amount_aed         NUMERIC(12,2),
  fixed_car_charging_aed    NUMERIC(12,2),
  base_amount_aed           NUMERIC(12,2) NOT NULL,
  commission_aed            NUMERIC(12,2),
  total_cash_submit_aed     NUMERIC(12,2),
  car_charging_diff_aed     NUMERIC(12,2),
  cash_diff_aed             NUMERIC(12,2),
  final_salary_aed          NUMERIC(12,2) NOT NULL,
  advance_deduction_aed     NUMERIC(12,2) NOT NULL DEFAULT 0,
  net_payable_aed           NUMERIC(12,2) NOT NULL,
  deductions_json           JSONB,
  slip_url                  TEXT,
  generated_by              UUID NOT NULL REFERENCES profiles(id),
  generated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(driver_id, period_month)
);

ALTER TABLE salaries ENABLE ROW LEVEL SECURITY;

-- ─── invoices ────────────────────────────────────────────────────────────────

CREATE TABLE invoices (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id      UUID NOT NULL REFERENCES drivers(id),
  invoice_no     TEXT NOT NULL UNIQUE,
  period_start   DATE NOT NULL,
  period_end     DATE NOT NULL,
  line_items_json JSONB NOT NULL DEFAULT '[]',
  total_aed      NUMERIC(12,2) NOT NULL,
  pdf_url        TEXT,
  generated_by   UUID NOT NULL REFERENCES profiles(id),
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;

-- ─── notifications ────────────────────────────────────────────────────────────

CREATE TABLE notifications (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  recipient_id UUID NOT NULL REFERENCES profiles(id),
  type         TEXT NOT NULL,
  subject      TEXT NOT NULL,
  body         TEXT NOT NULL,
  sent_at      TIMESTAMPTZ,
  resend_id    TEXT,
  status       TEXT NOT NULL DEFAULT 'pending',
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;

-- ─── settings ────────────────────────────────────────────────────────────────

CREATE TABLE settings (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  key         TEXT NOT NULL UNIQUE,
  value       TEXT NOT NULL,
  updated_by  UUID REFERENCES profiles(id),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default settings
INSERT INTO settings (key, value) VALUES
  ('pay_cycle_day',                   '25'),
  ('trip_cap_aed',                    '2000'),
  ('cash_shortfall_threshold_aed',    '500'),
  ('auto_salary_enabled',             'true'),
  ('company_name',                    'Fleet Management Co.'),
  ('company_address',                 'Dubai, UAE'),
  ('salary_target_high_aed',          '12300'),
  ('salary_fixed_car_high_aed',       '1600'),
  ('salary_target_low_aed',           '6600'),
  ('salary_fixed_car_low_aed',        '800');

ALTER TABLE settings ENABLE ROW LEVEL SECURITY;

-- ─── audit_log (append-only) ──────────────────────────────────────────────────

CREATE TABLE audit_log (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  actor_id      UUID NOT NULL REFERENCES profiles(id),
  actor_role    user_role NOT NULL,
  action        TEXT NOT NULL,
  entity_type   TEXT NOT NULL,
  entity_id     UUID,
  metadata_json JSONB,
  ip            TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;

-- ─── uber_trips ──────────────────────────────────────────────────────────────

CREATE TABLE uber_trips (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  driver_id    UUID NOT NULL REFERENCES drivers(id),
  uber_trip_id TEXT NOT NULL UNIQUE,
  raw_json     JSONB NOT NULL,
  synced_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  reconciled   BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE uber_trips ENABLE ROW LEVEL SECURITY;
