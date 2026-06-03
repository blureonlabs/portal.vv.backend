-- RESET: Delete all data except super_admin profile and settings
-- Runs once on deploy to clean the database for fresh start

-- Delete in dependency order (children first)

-- Salaries
DELETE FROM salaries;

-- Trip-related
DELETE FROM trip_platform_earnings;
DELETE FROM trips;
DELETE FROM uber_trips;

-- Finance
DELETE FROM expenses;
DELETE FROM cash_handovers;

-- Advances
DELETE FROM advances;

-- HR
DELETE FROM leave_requests;

-- Invoices
DELETE FROM invoice_counters;
DELETE FROM invoices;

-- Documents
DELETE FROM documents;

-- Vehicle-related
DELETE FROM vehicle_service;
DELETE FROM vehicle_assignments;
DELETE FROM vehicles;

-- Owners
DELETE FROM owners;

-- Drivers
DELETE FROM driver_edits;
DELETE FROM drivers;

-- Notifications & Comms
DELETE FROM notifications;
DELETE FROM broadcasts;

-- Audit log
DELETE FROM audit_log;

-- Invites
DELETE FROM invites;

-- Profiles: keep only super_admin
DELETE FROM profiles WHERE role != 'super_admin';

-- Reset settings to defaults
UPDATE settings SET value = '0.30' WHERE key = 'commission_rate';
UPDATE settings SET value = '12300' WHERE key = 'salary_target_high_aed';
UPDATE settings SET value = '6600' WHERE key = 'salary_target_low_aed';
UPDATE settings SET value = '800' WHERE key = 'salary_fixed_car_low_aed';
UPDATE settings SET value = '1600' WHERE key = 'salary_fixed_car_high_aed';
