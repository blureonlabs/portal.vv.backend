use actix_web::{web, HttpResponse};
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use super::dto::{AssignedVehicle, DayEarnings, DriverContextResponse, EarningsQuery, EarningsResponse};

fn require_driver(user: &CurrentUser) -> Result<(), AppError> {
    if user.role != Role::Driver {
        return Err(AppError::Forbidden("Driver role required".into()));
    }
    Ok(())
}

pub async fn get_me(
    user: CurrentUser,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
) -> Result<HttpResponse, AppError> {
    require_driver(&user)?;

    use crate::database::domain::DatabasePool;
    let pool = db.pg_pool();

    #[derive(sqlx::FromRow)]
    struct DriverRow {
        driver_id: Uuid,
        salary_type: String,
        nationality: Option<String>,
        self_entry_enabled: bool,
        full_name: String,
        email: String,
        vehicle_id: Option<Uuid>,
        plate_number: Option<String>,
        make: Option<String>,
        model: Option<String>,
        year: Option<i32>,
        color: Option<String>,
    }

    let row = sqlx::query_as::<_, DriverRow>(
        r#"
        SELECT
            d.id AS driver_id,
            d.salary_type::text AS salary_type,
            d.nationality,
            d.self_entry_enabled,
            p.full_name,
            p.email,
            v.id AS vehicle_id,
            v.plate_number,
            v.make,
            v.model,
            v.year,
            v.color
        FROM drivers d
        JOIN profiles p ON p.id = d.profile_id
        LEFT JOIN vehicles v ON v.assigned_driver_id = d.id AND v.is_active = true
        WHERE d.profile_id = $1 AND d.is_active = true
        "#,
    )
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Driver profile not found".into()))?;

    let vehicle = match row.vehicle_id {
        Some(id) => Some(AssignedVehicle {
            id,
            plate_number: row.plate_number.unwrap_or_default(),
            make: row.make.unwrap_or_default(),
            model: row.model.unwrap_or_default(),
            year: row.year.unwrap_or(0),
            color: row.color,
        }),
        None => None,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::ok(DriverContextResponse {
        profile_id: user.id,
        full_name: row.full_name,
        email: row.email,
        driver_id: row.driver_id,
        salary_type: row.salary_type,
        nationality: row.nationality,
        self_entry_enabled: row.self_entry_enabled,
        vehicle,
    })))
}

pub async fn get_earnings(
    user: CurrentUser,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<EarningsQuery>,
) -> Result<HttpResponse, AppError> {
    require_driver(&user)?;

    use crate::database::domain::DatabasePool;
    let pool = db.pg_pool();

    // Resolve driver_id from profile
    let driver_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM drivers WHERE profile_id = $1 AND is_active = true",
    )
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Driver profile not found".into()))?;

    // Parse month (default: current month)
    let today = Local::now().date_naive();
    let (year, month) = if let Some(m) = &query.month {
        let parts: Vec<&str> = m.split('-').collect();
        if parts.len() == 2 {
            let y = parts[0].parse::<i32>().unwrap_or(today.year());
            let mo = parts[1].parse::<u32>().unwrap_or(today.month());
            (y, mo)
        } else {
            (today.year(), today.month())
        }
    } else {
        (today.year(), today.month())
    };

    let from = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or(today);
    // Compute last day of the given month
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .ok_or_else(|| AppError::Internal("Date arithmetic overflow".into()))?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .ok_or_else(|| AppError::Internal("Date arithmetic overflow".into()))?
    };
    let last_day = next_month_first
        .pred_opt()
        .unwrap_or(today);
    let to = last_day.min(today);

    #[derive(sqlx::FromRow)]
    struct DayRow {
        trip_date: NaiveDate,
        cash_aed: Option<Decimal>,
        card_aed: Option<Decimal>,
        uber_cash_aed: Option<Decimal>,
        bolt_cash_aed: Option<Decimal>,
    }

    let rows = sqlx::query_as::<_, DayRow>(
        r#"
        SELECT
            trip_date,
            SUM(cash_aed) AS cash_aed,
            SUM(card_aed) AS card_aed,
            SUM(uber_cash_aed) AS uber_cash_aed,
            SUM(bolt_cash_aed) AS bolt_cash_aed
        FROM trips
        WHERE driver_id = $1 AND trip_date BETWEEN $2 AND $3 AND is_deleted = false
        GROUP BY trip_date
        ORDER BY trip_date
        "#,
    )
    .bind(driver_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    let days: Vec<DayEarnings> = rows
        .into_iter()
        .map(|r| {
            let cash = r.cash_aed.unwrap_or(Decimal::ZERO);
            let card = r.card_aed.unwrap_or(Decimal::ZERO);
            let uber = r.uber_cash_aed.unwrap_or(Decimal::ZERO);
            let bolt = r.bolt_cash_aed.unwrap_or(Decimal::ZERO);
            let other = uber + bolt;
            DayEarnings {
                date: r.trip_date,
                cash_aed: cash,
                card_aed: card,
                other_aed: other,
                total_aed: cash + card + other,
            }
        })
        .collect();

    let total_cash: Decimal = days.iter().map(|d| d.cash_aed).sum();
    let total_card: Decimal = days.iter().map(|d| d.card_aed).sum();
    let total_other: Decimal = days.iter().map(|d| d.other_aed).sum();

    Ok(HttpResponse::Ok().json(ApiResponse::ok(EarningsResponse {
        month: format!("{year:04}-{month:02}"),
        days,
        total_cash,
        total_card,
        total_other,
        grand_total: total_cash + total_card + total_other,
    })))
}
