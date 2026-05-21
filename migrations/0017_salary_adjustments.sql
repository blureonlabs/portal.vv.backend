-- Track adjustment chain: adjusted records point to the original
ALTER TABLE salaries ADD COLUMN IF NOT EXISTS adjusted_from_id UUID REFERENCES salaries(id);

-- Replace the strict UNIQUE with a partial unique index:
-- Only ONE "original" salary (adjusted_from_id IS NULL) per driver+month
ALTER TABLE salaries DROP CONSTRAINT IF EXISTS salaries_driver_id_period_month_key;
CREATE UNIQUE INDEX IF NOT EXISTS uq_salaries_driver_month_original
  ON salaries (driver_id, period_month)
  WHERE adjusted_from_id IS NULL;
