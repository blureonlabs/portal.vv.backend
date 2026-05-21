-- Fix RLS on documents table: ensure policy covers all roles
-- Drop and recreate to guarantee it works regardless of connection role
DROP POLICY IF EXISTS "backend_all" ON documents;
DROP POLICY IF EXISTS "allow_all" ON documents;

-- Create permissive policy for ALL roles (including pooler connections)
CREATE POLICY "allow_all_access" ON documents
    FOR ALL
    TO PUBLIC
    USING (true)
    WITH CHECK (true);
