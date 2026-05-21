-- License number on drivers (Section 3.1)
ALTER TABLE drivers ADD COLUMN IF NOT EXISTS license_number TEXT;

-- Document metadata (Section 3.4)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS document_number TEXT;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS issue_date DATE;
