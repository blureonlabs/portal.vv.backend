-- Payment tracking for salaries
CREATE TYPE salary_status AS ENUM ('draft', 'approved', 'paid');

ALTER TABLE salaries ADD COLUMN IF NOT EXISTS status salary_status NOT NULL DEFAULT 'draft';
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS approved_by UUID REFERENCES profiles(id);
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS approved_at TIMESTAMPTZ;
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS payment_date DATE;
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS payment_mode TEXT;  -- 'bank_transfer', 'cash', 'cheque'
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS payment_reference TEXT;
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS paid_at TIMESTAMPTZ;
