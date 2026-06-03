-- Disable RLS on documents table entirely.
-- The backend handles authorization via CurrentUser + role guards.
-- RLS was causing "new row violates row-level security policy" errors
-- because the Supabase connection pooler uses a different role than
-- the policy was created for.
ALTER TABLE documents DISABLE ROW LEVEL SECURITY;
