use std::sync::Arc;

use chrono::NaiveDate;
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::error::AppError;
use crate::common::types::Role;
use crate::trip::domain::{
    entity::{CreateTrip, CsvPreviewRow, MonthlyEarnings, Trip, TripSource},
    repository::TripRepository,
};

pub struct TripService {
    pub repo: Arc<dyn TripRepository>,
    pub audit: Arc<AuditService>,
}

impl TripService {
    pub fn new(repo: Arc<dyn TripRepository>, audit: Arc<AuditService>) -> Self {
        Self { repo, audit }
    }

    pub async fn list(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Trip>, AppError> {
        // Drivers can only see their own trips
        let effective_driver_id = if *actor_role == Role::Driver {
            actor_driver_id
        } else {
            driver_id
        };
        self.repo.list(effective_driver_id, from, to).await
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Uuid,
        vehicle_id: Option<Uuid>,
        trip_date: NaiveDate,
        cash_aed: Decimal,
        uber_cash_aed: Decimal,
        bolt_cash_aed: Decimal,
        card_aed: Decimal,
        notes: Option<String>,
    ) -> Result<(Trip, Option<String>), AppError> {
        // Drivers can only enter their own trips, and only if self_entry_enabled
        if *actor_role == Role::Driver {
            let own_driver_id = actor_driver_id
                .ok_or_else(|| AppError::Forbidden("No driver record linked to your account".into()))?;

            if own_driver_id != driver_id {
                return Err(AppError::Forbidden("Drivers can only enter their own trips".into()));
            }

            let enabled = self.repo.driver_self_entry_enabled(own_driver_id).await?;
            if !enabled {
                return Err(AppError::Forbidden(
                    "Self-entry is disabled for your account. Contact your admin.".into(),
                ));
            }
        }

        // Daily cap check
        let cap = self.repo.get_trip_cap().await?;
        let existing = self.repo.daily_total(driver_id, trip_date).await?;
        let new_total = cash_aed + uber_cash_aed + bolt_cash_aed + card_aed;

        if existing + new_total > cap {
            return Err(AppError::BadRequest(format!(
                "Daily cap of AED {cap} exceeded (current: AED {existing}, adding: AED {new_total})"
            )));
        }

        let trip = self.repo
            .create(CreateTrip {
                driver_id,
                vehicle_id,
                entered_by: actor_id,
                trip_date,
                cash_aed,
                uber_cash_aed,
                bolt_cash_aed,
                card_aed,
                source: TripSource::Manual,
                notes,
            })
            .await?;

        self.audit.log(actor_id, actor_role, "trip", Some(trip.id), "trip.created",
            Some(serde_json::json!({ "driver_id": driver_id, "trip_date": trip_date }))).await?;

        // Conflict detection: check if another trip for the same driver+date
        // was entered from a different source. Admin entries take priority.
        let existing_trips = self.repo.find_by_driver_and_date(driver_id, trip_date).await?;
        let new_source = &trip.source;
        let conflict_warning = existing_trips.iter()
            .filter(|t| t.id != trip.id && &t.source != new_source)
            .map(|t| format!(
                "A trip from source '{:?}' already exists for driver {} on {}. Admin entry takes priority.",
                t.source, driver_id, trip_date
            ))
            .next();

        Ok((trip, conflict_warning))
    }

    pub async fn update(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        trip_id: Uuid,
        driver_id: Uuid,
        vehicle_id: Option<Uuid>,
        trip_date: NaiveDate,
        cash_aed: Decimal,
        uber_cash_aed: Decimal,
        bolt_cash_aed: Decimal,
        card_aed: Decimal,
        notes: Option<String>,
    ) -> Result<Trip, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can edit trips".into())),
        }

        // Daily cap check — exclude the current trip from existing total
        let cap = self.repo.get_trip_cap().await?;
        let existing = self.repo.daily_total_excluding(driver_id, trip_date, trip_id).await?;
        let new_total = cash_aed + uber_cash_aed + bolt_cash_aed + card_aed;

        if existing + new_total > cap {
            return Err(AppError::BadRequest(format!(
                "Daily cap of AED {cap} exceeded (current: AED {existing}, adding: AED {new_total})"
            )));
        }

        let trip = self.repo
            .update(trip_id, CreateTrip {
                driver_id,
                vehicle_id,
                entered_by: actor_id,
                trip_date,
                cash_aed,
                uber_cash_aed,
                bolt_cash_aed,
                card_aed,
                source: TripSource::Manual,
                notes,
            })
            .await?;

        self.audit.log(actor_id, actor_role, "trip", Some(trip_id), "trip.updated",
            Some(serde_json::json!({ "driver_id": driver_id, "trip_date": trip_date }))).await?;

        Ok(trip)
    }

    pub async fn monthly_earnings(
        &self,
        driver_id: Uuid,
        month: NaiveDate,
    ) -> Result<MonthlyEarnings, AppError> {
        self.repo.monthly_earnings(driver_id, month).await
    }

    pub async fn delete(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        id: Uuid,
    ) -> Result<(), AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can delete trips".into())),
        }
        self.repo.soft_delete(id).await?;
        self.audit.log(actor_id, actor_role, "trip", Some(id), "trip.deleted", None).await?;
        Ok(())
    }

    /// Parse CSV, validate each row against daily cap, return preview.
    /// CSV format: date,cash_aed,uber_cash_aed,bolt_cash_aed,card_aed,notes (header required)
    pub async fn csv_preview(
        &self,
        driver_id: Uuid,
        csv_content: &str,
    ) -> Result<Vec<CsvPreviewRow>, AppError> {
        let cap = self.repo.get_trip_cap().await?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_content.as_bytes());

        let mut rows: Vec<CsvPreviewRow> = Vec::new();

        // We'll accumulate daily totals as we parse to correctly flag cap warnings
        // (considering rows parsed so far + existing DB rows)
        let mut pending_totals: std::collections::HashMap<NaiveDate, Decimal> = std::collections::HashMap::new();

        for (i, result) in rdr.records().enumerate() {
            let row_num = i + 1;
            match result {
                Err(e) => {
                    rows.push(CsvPreviewRow {
                        row_num,
                        trip_date: String::new(),
                        cash_aed: Decimal::ZERO,
                        uber_cash_aed: Decimal::ZERO,
                        bolt_cash_aed: Decimal::ZERO,
                        card_aed: Decimal::ZERO,
                        notes: None,
                        error: Some(format!("CSV parse error: {e}")),
                        cap_warning: None,
                    });
                }
                Ok(record) => {
                    let parse_result = parse_csv_row(&record);
                    match parse_result {
                        Err(e) => {
                            rows.push(CsvPreviewRow {
                                row_num,
                                trip_date: record.get(0).unwrap_or("").to_string(),
                                cash_aed: Decimal::ZERO,
                                uber_cash_aed: Decimal::ZERO,
                                bolt_cash_aed: Decimal::ZERO,
                                card_aed: Decimal::ZERO,
                                notes: None,
                                error: Some(e),
                                cap_warning: None,
                            });
                        }
                        Ok((date, cash, uber_cash, bolt_cash, card, notes)) => {
                            let row_total = cash + uber_cash + bolt_cash + card;
                            let db_total = self.repo.daily_total(driver_id, date).await.unwrap_or(Decimal::ZERO);
                            let pending = *pending_totals.get(&date).unwrap_or(&Decimal::ZERO);
                            let combined = db_total + pending + row_total;

                            let cap_warning = if combined > cap {
                                Some(format!(
                                    "Would exceed daily cap of AED {cap} (total would be AED {combined})"
                                ))
                            } else {
                                None
                            };

                            *pending_totals.entry(date).or_insert(Decimal::ZERO) += row_total;

                            rows.push(CsvPreviewRow {
                                row_num,
                                trip_date: date.to_string(),
                                cash_aed: cash,
                                uber_cash_aed: uber_cash,
                                bolt_cash_aed: bolt_cash,
                                card_aed: card,
                                notes,
                                error: None,
                                cap_warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(rows)
    }

    /// Bulk import validated CSV rows (skips rows with errors, inserts valid ones).
    pub async fn csv_import(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        driver_id: Uuid,
        preview_rows: Vec<CsvPreviewRow>,
    ) -> Result<Vec<Trip>, AppError> {
        let to_insert: Vec<CreateTrip> = preview_rows
            .into_iter()
            .filter(|r| r.error.is_none())
            .filter_map(|r| {
                let date = r.trip_date.parse::<NaiveDate>().ok()?;
                Some(CreateTrip {
                    driver_id,
                    vehicle_id: None,
                    entered_by: actor_id,
                    trip_date: date,
                    cash_aed: r.cash_aed,
                    uber_cash_aed: r.uber_cash_aed,
                    bolt_cash_aed: r.bolt_cash_aed,
                    card_aed: r.card_aed,
                    source: TripSource::CsvImport,
                    notes: r.notes,
                })
            })
            .collect();

        let count = to_insert.len();
        let trips = self.repo.bulk_insert(to_insert).await?;

        self.audit.log(actor_id, actor_role, "trip", None, "trip.csv_imported",
            Some(serde_json::json!({ "driver_id": driver_id, "rows_imported": count }))).await?;

        Ok(trips)
    }
}

fn parse_csv_row(
    record: &csv::StringRecord,
) -> Result<(NaiveDate, Decimal, Decimal, Decimal, Decimal, Option<String>), String> {
    use crate::common::validation::validate_amount;

    let date_str = record.get(0).unwrap_or("").trim();
    let date = date_str
        .parse::<NaiveDate>()
        .map_err(|_| format!("Invalid date '{date_str}' (expected YYYY-MM-DD)"))?;

    let cash: Decimal = record
        .get(1)
        .unwrap_or("0")
        .trim()
        .parse()
        .map_err(|_| "Invalid cash_aed value".to_string())?;
    validate_amount("cash_aed", cash).map_err(|e| e.to_string())?;

    let uber_cash: Decimal = record
        .get(2)
        .unwrap_or("0")
        .trim()
        .parse()
        .map_err(|_| "Invalid uber_cash_aed value".to_string())?;
    validate_amount("uber_cash_aed", uber_cash).map_err(|e| e.to_string())?;

    let bolt_cash: Decimal = record
        .get(3)
        .unwrap_or("0")
        .trim()
        .parse()
        .map_err(|_| "Invalid bolt_cash_aed value".to_string())?;
    validate_amount("bolt_cash_aed", bolt_cash).map_err(|e| e.to_string())?;

    let card: Decimal = record
        .get(4)
        .unwrap_or("0")
        .trim()
        .parse()
        .map_err(|_| "Invalid card_aed value".to_string())?;
    validate_amount("card_aed", card).map_err(|e| e.to_string())?;

    let notes = record
        .get(5)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    Ok((date, cash, uber_cash, bolt_cash, card, notes))
}
