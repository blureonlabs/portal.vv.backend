use rust_decimal::Decimal;
use uuid::Uuid;

/// Domain events emitted by feature modules.
/// Consumed by `notification` (email dispatch) and `audit` (immutable log).
/// Features never call notification/audit directly — they emit events only.
#[derive(Debug, Clone)]
pub enum DomainEvent {
    // Auth
    InviteSent        { invitee_email: String, invited_by: Uuid },
    InviteResent      { invitee_email: String, invited_by: Uuid },
    InviteAccepted    { user_id: Uuid },
    InviteRevoked     { invitee_email: String, revoked_by: Uuid },
    PasswordResetRequested { user_id: Uuid },
    PasswordResetCompleted { user_id: Uuid },

    // Driver
    DriverCreated     { driver_id: Uuid, by: Uuid },
    DriverEdited      { driver_id: Uuid, by: Uuid, field: String, old_val: String, new_val: String },
    DriverDeactivated { driver_id: Uuid, by: Uuid },
    DriverReactivated { driver_id: Uuid, by: Uuid },

    // Vehicle
    VehicleCreated    { vehicle_id: Uuid, by: Uuid },
    VehicleAssigned   { vehicle_id: Uuid, driver_id: Uuid, by: Uuid },
    VehicleUnassigned { vehicle_id: Uuid, driver_id: Uuid, by: Uuid },
    VehicleServiceLogged    { vehicle_id: Uuid, by: Uuid },
    VehicleInsuranceAlert   { vehicle_id: Uuid, days_until_expiry: i32 },
    VehicleServiceDue       { vehicle_id: Uuid },

    // Trip
    TripCreated       { trip_id: Uuid, driver_id: Uuid, by: Uuid },
    TripEdited        { trip_id: Uuid, by: Uuid },
    TripDeleted       { trip_id: Uuid, by: Uuid },
    TripCsvImported   { row_count: i32, by: Uuid },

    // Finance
    ExpenseAdded      { expense_id: Uuid, driver_id: Uuid, by: Uuid },
    CashHandoverRecorded { driver_id: Uuid, amount: Decimal, by: Uuid },

    // Advance
    AdvanceRequested  { advance_id: Uuid, driver_id: Uuid, amount: Decimal },
    AdvanceApproved   { advance_id: Uuid, driver_id: Uuid, by: Uuid },
    AdvanceRejected   { advance_id: Uuid, driver_id: Uuid, reason: String, by: Uuid },
    AdvancePaid       { advance_id: Uuid, driver_id: Uuid, by: Uuid },

    // HR
    LeaveRequested    { request_id: Uuid, driver_id: Uuid },
    LeaveApproved     { request_id: Uuid, driver_id: Uuid, by: Uuid },
    LeaveRejected     { request_id: Uuid, driver_id: Uuid, reason: Option<String>, by: Uuid },

    // Salary
    SalaryGenerated   { salary_id: Uuid, driver_id: Uuid, period: String, by: Uuid },

    // Invoice
    InvoiceGenerated  { invoice_id: Uuid, driver_id: Uuid, by: Uuid },

    // Settings
    SettingChanged    { key: String, old_val: String, new_val: String, by: Uuid },
}
