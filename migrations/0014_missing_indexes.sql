CREATE INDEX IF NOT EXISTS idx_notifications_recipient_id ON notifications(recipient_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor_id ON audit_log(actor_id);
CREATE INDEX IF NOT EXISTS idx_leave_requests_driver_id ON leave_requests(driver_id);
CREATE INDEX IF NOT EXISTS idx_advances_driver_id ON advances(driver_id);
