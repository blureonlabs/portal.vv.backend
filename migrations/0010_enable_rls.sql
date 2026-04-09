-- Enable RLS on all tables that were flagged by Supabase security advisor
-- Our Rust backend handles auth via JWT middleware, but RLS adds defense-in-depth

ALTER TABLE _sqlx_migrations ENABLE ROW LEVEL SECURITY;
ALTER TABLE broadcasts ENABLE ROW LEVEL SECURITY;
ALTER TABLE owners ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;

-- Allow the postgres role (used by our backend connection) full access
CREATE POLICY "Backend full access" ON broadcasts FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "Backend full access" ON owners FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "Backend full access" ON documents FOR ALL USING (true) WITH CHECK (true);
CREATE POLICY "Backend full access" ON _sqlx_migrations FOR ALL USING (true) WITH CHECK (true);
