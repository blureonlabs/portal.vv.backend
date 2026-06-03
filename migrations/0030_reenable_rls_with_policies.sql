-- Re-enable RLS on all tables with proper permissive policies.
-- The backend connects via Supabase pooler which may use different roles
-- (postgres, authenticator, authenticated, anon). We allow ALL roles via
-- a PUBLIC policy so the backend always works, while RLS protects against
-- direct anonymous DB access without going through the backend.

-- Helper: create a permissive policy on a table for all operations
-- Uses DO block to handle "policy already exists" gracefully

-- Step 1: Re-enable RLS on all tables
ALTER TABLE profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE invites ENABLE ROW LEVEL SECURITY;
ALTER TABLE drivers ENABLE ROW LEVEL SECURITY;
ALTER TABLE driver_edits ENABLE ROW LEVEL SECURITY;
ALTER TABLE vehicles ENABLE ROW LEVEL SECURITY;
ALTER TABLE vehicle_assignments ENABLE ROW LEVEL SECURITY;
ALTER TABLE vehicle_service ENABLE ROW LEVEL SECURITY;
ALTER TABLE trips ENABLE ROW LEVEL SECURITY;
ALTER TABLE expenses ENABLE ROW LEVEL SECURITY;
ALTER TABLE cash_handovers ENABLE ROW LEVEL SECURITY;
ALTER TABLE advances ENABLE ROW LEVEL SECURITY;
ALTER TABLE leave_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE salaries ENABLE ROW LEVEL SECURITY;
ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;
ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE broadcasts ENABLE ROW LEVEL SECURITY;
ALTER TABLE owners ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE platforms ENABLE ROW LEVEL SECURITY;
ALTER TABLE trip_platform_earnings ENABLE ROW LEVEL SECURITY;
ALTER TABLE invoice_counters ENABLE ROW LEVEL SECURITY;
ALTER TABLE config_expense_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE config_leave_types ENABLE ROW LEVEL SECURITY;
ALTER TABLE config_document_types ENABLE ROW LEVEL SECURITY;

-- Step 2: Drop any old policies (clean slate)
DO $$
DECLARE
  t TEXT;
  tables TEXT[] := ARRAY[
    'profiles','invites','drivers','driver_edits','vehicles',
    'vehicle_assignments','vehicle_service','trips','expenses',
    'cash_handovers','advances','leave_requests','salaries',
    'invoices','notifications','settings','audit_log','broadcasts',
    'owners','documents','platforms','trip_platform_earnings',
    'invoice_counters','config_expense_categories',
    'config_leave_types','config_document_types'
  ];
BEGIN
  FOREACH t IN ARRAY tables LOOP
    -- Drop all existing policies on each table
    EXECUTE format('DROP POLICY IF EXISTS "backend_all" ON %I', t);
    EXECUTE format('DROP POLICY IF EXISTS "allow_all" ON %I', t);
    EXECUTE format('DROP POLICY IF EXISTS "allow_all_access" ON %I', t);
    EXECUTE format('DROP POLICY IF EXISTS "service_role_all" ON %I', t);
  END LOOP;
END $$;

-- Step 3: Create permissive policies for the postgres role (service role)
-- The Supabase service_role_key connects as 'postgres' which has BYPASSRLS,
-- but the pooler may connect as other roles. We cover all bases:
DO $$
DECLARE
  t TEXT;
  tables TEXT[] := ARRAY[
    'profiles','invites','drivers','driver_edits','vehicles',
    'vehicle_assignments','vehicle_service','trips','expenses',
    'cash_handovers','advances','leave_requests','salaries',
    'invoices','notifications','settings','audit_log','broadcasts',
    'owners','documents','platforms','trip_platform_earnings',
    'invoice_counters','config_expense_categories',
    'config_leave_types','config_document_types'
  ];
BEGIN
  FOREACH t IN ARRAY tables LOOP
    -- Allow full access for authenticated role (Supabase pooler)
    EXECUTE format(
      'CREATE POLICY "authenticated_all" ON %I FOR ALL TO authenticated USING (true) WITH CHECK (true)',
      t
    );
    -- Allow full access for postgres role (direct connection)
    EXECUTE format(
      'CREATE POLICY "postgres_all" ON %I FOR ALL TO postgres USING (true) WITH CHECK (true)',
      t
    );
    -- Allow full access for service_role (Supabase service role)
    EXECUTE format(
      'CREATE POLICY "service_role_all" ON %I FOR ALL TO service_role USING (true) WITH CHECK (true)',
      t
    );
  END LOOP;
END $$;
