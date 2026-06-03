use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::auth::presentation::handlers::require_role;

#[derive(Debug, Deserialize)]
pub struct LedgerQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LedgerEntry {
    pub date: NaiveDate,
    pub entry_type: String,
    pub description: String,
    pub debit: Decimal,
    pub credit: Decimal,
}

pub async fn get_driver_ledger(
    user: CurrentUser,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    path: web::Path<Uuid>,
    query: web::Query<LedgerQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;

    let driver_id = path.into_inner();
    let from = query.from.as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().date_naive() - chrono::Duration::days(90));
    let to = query.to.as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let pool = db.pg_pool();

    // Union query across trips, advances, salaries, cash_handovers
    let entries = sqlx::query_as::<_, LedgerEntry>(
        r#"
        SELECT * FROM (
            -- Trips (credit)
            SELECT trip_date AS date, 'trip'::text AS entry_type,
                   'Daily trip earnings'::text AS description,
                   0::numeric AS debit,
                   (cash_aed + uber_cash_aed + bolt_cash_aed + card_aed) AS credit
            FROM trips
            WHERE driver_id = $1 AND trip_date BETWEEN $2 AND $3 AND is_deleted = false

            UNION ALL

            -- Advances paid (debit)
            SELECT COALESCE(payment_date, created_at::date) AS date, 'advance'::text AS entry_type,
                   ('Advance: ' || LEFT(reason, 50))::text AS description,
                   amount_aed AS debit,
                   0::numeric AS credit
            FROM advances
            WHERE driver_id = $1 AND status = 'paid'
              AND COALESCE(payment_date, created_at::date) BETWEEN $2 AND $3

            UNION ALL

            -- Salary (credit)
            SELECT period_month AS date, 'salary'::text AS entry_type,
                   ('Salary: ' || TO_CHAR(period_month, 'Mon YYYY'))::text AS description,
                   0::numeric AS debit,
                   net_payable_aed AS credit
            FROM salaries
            WHERE driver_id = $1 AND period_month BETWEEN $2 AND $3
              AND status IN ('approved', 'paid')
              AND adjusted_from_id IS NULL

            UNION ALL

            -- Cash handovers (debit — driver submitting cash)
            SELECT submitted_at::date AS date, 'handover'::text AS entry_type,
                   'Cash handover'::text AS description,
                   amount_aed AS debit,
                   0::numeric AS credit
            FROM cash_handovers
            WHERE driver_id = $1 AND submitted_at::date BETWEEN $2 AND $3
        ) ledger
        ORDER BY date DESC, entry_type
        "#
    )
    .bind(driver_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::ok(entries)))
}
