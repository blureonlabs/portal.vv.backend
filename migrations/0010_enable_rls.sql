-- Enable RLS on tables flagged by Supabase security advisor
-- Do NOT enable RLS on _sqlx_migrations (breaks sqlx migration runner)
ALTER TABLE broadcasts ENABLE ROW LEVEL SECURITY;
ALTER TABLE owners ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;

-- Allow all access (our backend uses service role, not anon)
DO $$ BEGIN
  CREATE POLICY "backend_all" ON broadcasts FOR ALL USING (true) WITH CHECK (true);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN
  CREATE POLICY "backend_all" ON owners FOR ALL USING (true) WITH CHECK (true);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN
  CREATE POLICY "backend_all" ON documents FOR ALL USING (true) WITH CHECK (true);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
