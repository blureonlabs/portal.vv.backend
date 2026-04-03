-- Performance indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_trips_driver_date ON trips(driver_id, trip_date);
CREATE INDEX IF NOT EXISTS idx_advances_driver_status ON advances(driver_id, status);
CREATE INDEX IF NOT EXISTS idx_vehicles_insurance_expiry ON vehicles(insurance_expiry) WHERE insurance_expiry IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_leave_requests_driver_status ON leave_requests(driver_id, status);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_salaries_driver_period ON salaries(driver_id, period_month);
