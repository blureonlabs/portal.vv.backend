use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::report::domain::entity::{
    AdvanceReportRow, CashFlowRow, DashboardKpis, DayRevenue, DriverPerfRow, DriverSummaryRow,
    FinanceReport, FinanceSummaryRow, InsuranceAlert, LeaveReportRow, SalaryReportRow, TripDetailRow,
};

pub struct ReportService {
    pool: sqlx::PgPool,
}

impl ReportService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn driver_summary(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DriverSummaryRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            driver_id: Uuid,
            driver_name: String,
            trips_count: Option<i64>,
            total_revenue_aed: Option<Decimal>,
            total_expenses_aed: Option<Decimal>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                d.id AS driver_id,
                p.full_name AS driver_name,
                COALESCE(t.trips_count, 0) AS trips_count,
                COALESCE(t.total_revenue_aed, 0) AS total_revenue_aed,
                COALESCE(e.total_expenses_aed, 0) AS total_expenses_aed
            FROM drivers d
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN (
                SELECT driver_id,
                       COUNT(*) AS trips_count,
                       SUM(cash_aed + card_aed + other_aed) AS total_revenue_aed
                FROM trips
                WHERE trip_date BETWEEN $1 AND $2 AND is_deleted = false
                GROUP BY driver_id
            ) t ON t.driver_id = d.id
            LEFT JOIN (
                SELECT driver_id, SUM(amount_aed) AS total_expenses_aed
                FROM expenses
                WHERE date BETWEEN $1 AND $2
                GROUP BY driver_id
            ) e ON e.driver_id = d.id
            ORDER BY p.full_name
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let revenue = r.total_revenue_aed.unwrap_or(Decimal::ZERO);
                let expenses = r.total_expenses_aed.unwrap_or(Decimal::ZERO);
                DriverSummaryRow {
                    driver_id: r.driver_id,
                    driver_name: r.driver_name,
                    trips_count: r.trips_count.unwrap_or(0),
                    total_revenue_aed: revenue,
                    total_expenses_aed: expenses,
                    net_aed: revenue - expenses,
                }
            })
            .collect())
    }

    pub async fn trip_detail(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        driver_id: Option<Uuid>,
    ) -> Result<Vec<TripDetailRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            trip_id: Uuid,
            driver_name: String,
            trip_date: NaiveDate,
            cash_aed: Decimal,
            card_aed: Decimal,
            other_aed: Decimal,
            notes: Option<String>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                t.id AS trip_id,
                p.full_name AS driver_name,
                t.trip_date,
                t.cash_aed,
                t.card_aed,
                t.other_aed,
                t.notes
            FROM trips t
            JOIN drivers d ON d.id = t.driver_id
            JOIN profiles p ON p.id = d.profile_id
            WHERE t.is_deleted = false
              AND t.trip_date BETWEEN $1 AND $2
              AND ($3::uuid IS NULL OR t.driver_id = $3)
            ORDER BY t.trip_date DESC, t.created_at DESC
            "#,
        )
        .bind(from)
        .bind(to)
        .bind(driver_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let total = r.cash_aed + r.card_aed + r.other_aed;
                TripDetailRow {
                    trip_id: r.trip_id,
                    driver_name: r.driver_name,
                    trip_date: r.trip_date,
                    cash_aed: r.cash_aed,
                    card_aed: r.card_aed,
                    other_aed: r.other_aed,
                    total_aed: total,
                    notes: r.notes,
                }
            })
            .collect())
    }

    pub async fn finance_summary(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<FinanceReport, AppError> {
        // Revenue totals from trips
        #[derive(sqlx::FromRow)]
        struct RevenueRow {
            cash_total: Option<Decimal>,
            card_total: Option<Decimal>,
            other_total: Option<Decimal>,
        }
        let rev = sqlx::query_as::<_, RevenueRow>(
            "SELECT SUM(cash_aed) AS cash_total, SUM(card_aed) AS card_total, SUM(other_aed) AS other_total \
             FROM trips WHERE trip_date BETWEEN $1 AND $2 AND is_deleted = false",
        )
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        // Expense totals by category
        #[derive(sqlx::FromRow)]
        struct ExpCatRow {
            category: String,
            total_aed: Option<Decimal>,
        }
        let exp_cats = sqlx::query_as::<_, ExpCatRow>(
            "SELECT category::text AS category, SUM(amount_aed) AS total_aed \
             FROM expenses WHERE date BETWEEN $1 AND $2 \
             GROUP BY category ORDER BY category",
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        // Handovers total
        #[derive(sqlx::FromRow)]
        struct HandoverRow {
            total: Option<Decimal>,
        }
        let handover = sqlx::query_as::<_, HandoverRow>(
            "SELECT SUM(amount_aed) AS total FROM cash_handovers \
             WHERE submitted_at::date BETWEEN $1 AND $2",
        )
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        let cash = rev.cash_total.unwrap_or(Decimal::ZERO);
        let card = rev.card_total.unwrap_or(Decimal::ZERO);
        let other = rev.other_total.unwrap_or(Decimal::ZERO);
        let revenue_total = cash + card + other;

        let expense_rows: Vec<FinanceSummaryRow> = exp_cats
            .into_iter()
            .map(|r| FinanceSummaryRow {
                category: r.category,
                total_aed: r.total_aed.unwrap_or(Decimal::ZERO),
            })
            .collect();

        let total_expenses: Decimal = expense_rows.iter().map(|r| r.total_aed).sum();
        let total_handovers = handover.total.unwrap_or(Decimal::ZERO);

        Ok(FinanceReport {
            trip_revenue_cash: cash,
            trip_revenue_card: card,
            trip_revenue_other: other,
            trip_revenue_total: revenue_total,
            expense_by_category: expense_rows,
            total_expenses,
            total_handovers,
            net_aed: revenue_total - total_expenses,
        })
    }

    pub async fn dashboard(&self) -> Result<DashboardKpis, AppError> {
        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
        let thirty_days_ago = today - chrono::Duration::days(29);

        // Run ALL queries concurrently to minimize round-trip latency
        #[derive(sqlx::FromRow)]
        struct MtdRow { revenue: Option<Decimal>, trips: Option<i64> }
        #[derive(sqlx::FromRow)]
        struct CountsRow {
            active_drivers: Option<i64>,
            active_vehicles: Option<i64>,
            pending_advances: Option<i64>,
            pending_leave: Option<i64>,
        }
        #[derive(sqlx::FromRow)]
        struct InsRow { vehicle_id: Uuid, plate_number: String, insurance_expiry: NaiveDate }
        #[derive(sqlx::FromRow)]
        struct TopRow { driver_id: Uuid, driver_name: String, trips_count: Option<i64>, revenue_aed: Option<Decimal> }
        #[derive(sqlx::FromRow)]
        struct DayRow { date: NaiveDate, revenue_aed: Option<Decimal>, trips_count: Option<i64> }
        #[derive(sqlx::FromRow)]
        struct ExpMtdRow { total: Option<Decimal> }

        let pool = &self.pool;

        // Fire all 6 queries concurrently
        let (mtd, counts, ins_rows, top_rows, day_rows, exp_mtd) = tokio::try_join!(
            sqlx::query_as::<_, MtdRow>(
                "SELECT SUM(cash_aed + card_aed + other_aed) AS revenue, COUNT(*) AS trips \
                 FROM trips WHERE trip_date BETWEEN $1 AND $2 AND is_deleted = false"
            ).bind(month_start).bind(today).fetch_one(pool),

            sqlx::query_as::<_, CountsRow>(
                "SELECT \
                   (SELECT COUNT(*) FROM drivers WHERE is_active = true) AS active_drivers, \
                   (SELECT COUNT(*) FROM vehicles WHERE status != 'inactive') AS active_vehicles, \
                   (SELECT COUNT(*) FROM advances WHERE status = 'pending') AS pending_advances, \
                   (SELECT COUNT(*) FROM leave_requests WHERE status = 'pending') AS pending_leave"
            ).fetch_one(pool),

            sqlx::query_as::<_, InsRow>(
                "SELECT id AS vehicle_id, plate_number, insurance_expiry \
                 FROM vehicles WHERE insurance_expiry IS NOT NULL \
                   AND insurance_expiry <= $1 + INTERVAL '30 days' \
                 ORDER BY insurance_expiry"
            ).bind(today).fetch_all(pool),

            sqlx::query_as::<_, TopRow>(
                "SELECT t.driver_id, p.full_name AS driver_name, COUNT(*) AS trips_count, \
                        SUM(t.cash_aed + t.card_aed + t.other_aed) AS revenue_aed \
                 FROM trips t JOIN drivers d ON d.id = t.driver_id JOIN profiles p ON p.id = d.profile_id \
                 WHERE t.trip_date BETWEEN $1 AND $2 AND t.is_deleted = false \
                 GROUP BY t.driver_id, p.full_name ORDER BY revenue_aed DESC LIMIT 5"
            ).bind(month_start).bind(today).fetch_all(pool),

            sqlx::query_as::<_, DayRow>(
                "SELECT trip_date AS date, SUM(cash_aed + card_aed + other_aed) AS revenue_aed, COUNT(*) AS trips_count \
                 FROM trips WHERE trip_date BETWEEN $1 AND $2 AND is_deleted = false \
                 GROUP BY trip_date ORDER BY trip_date"
            ).bind(thirty_days_ago).bind(today).fetch_all(pool),

            sqlx::query_as::<_, ExpMtdRow>(
                "SELECT SUM(amount_aed) AS total FROM expenses WHERE date BETWEEN $1 AND $2"
            ).bind(month_start).bind(today).fetch_one(pool),
        )?;

        let insurance_expiring_soon = ins_rows.into_iter().map(|r| {
            let days_left = (r.insurance_expiry - today).num_days();
            let is_expired = r.insurance_expiry < today;
            InsuranceAlert { vehicle_id: r.vehicle_id, plate_number: r.plate_number, insurance_expiry: r.insurance_expiry, days_left, is_expired }
        }).collect();

        let top_drivers = top_rows.into_iter().map(|r| DriverPerfRow {
            driver_id: r.driver_id,
            driver_name: r.driver_name,
            trips_count: r.trips_count.unwrap_or(0),
            revenue_aed: r.revenue_aed.unwrap_or(Decimal::ZERO),
        }).collect();

        let revenue_trend = day_rows.into_iter().map(|r| DayRevenue {
            date: r.date,
            revenue_aed: r.revenue_aed.unwrap_or(Decimal::ZERO),
            trips_count: r.trips_count.unwrap_or(0),
        }).collect();

        let revenue_mtd = mtd.revenue.unwrap_or(Decimal::ZERO);
        let total_expenses_mtd = exp_mtd.total.unwrap_or(Decimal::ZERO);

        Ok(DashboardKpis {
            revenue_mtd,
            trips_mtd: mtd.trips.unwrap_or(0),
            active_drivers: counts.active_drivers.unwrap_or(0),
            active_vehicles: counts.active_vehicles.unwrap_or(0),
            pending_advances: counts.pending_advances.unwrap_or(0),
            pending_leave: counts.pending_leave.unwrap_or(0),
            total_expenses_mtd,
            net_profit: revenue_mtd - total_expenses_mtd,
            insurance_expiring_soon,
            top_drivers,
            revenue_trend,
        })
    }

    pub async fn advance_report(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<AdvanceReportRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            driver_name: String,
            total_requested: Option<Decimal>,
            total_approved: Option<Decimal>,
            total_paid: Option<Decimal>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                p.full_name AS driver_name,
                COALESCE(SUM(a.amount_aed), 0) AS total_requested,
                COALESCE(SUM(CASE WHEN a.status IN ('approved', 'paid') THEN a.amount_aed ELSE 0 END), 0) AS total_approved,
                COALESCE(SUM(CASE WHEN a.status = 'paid' THEN a.amount_aed ELSE 0 END), 0) AS total_paid
            FROM drivers d
            JOIN profiles p ON p.id = d.profile_id
            JOIN advances a ON a.driver_id = d.id
            WHERE a.created_at::date BETWEEN $1 AND $2
            GROUP BY p.full_name
            ORDER BY p.full_name
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let requested = r.total_requested.unwrap_or(Decimal::ZERO);
                let approved = r.total_approved.unwrap_or(Decimal::ZERO);
                let paid = r.total_paid.unwrap_or(Decimal::ZERO);
                AdvanceReportRow {
                    driver_name: r.driver_name,
                    total_requested: requested,
                    total_approved: approved,
                    total_paid: paid,
                    outstanding_balance: approved - paid,
                }
            })
            .collect())
    }

    pub async fn cash_flow_report(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<CashFlowRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            driver_name: String,
            total_cash_received: Option<Decimal>,
            total_cash_submitted: Option<Decimal>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                p.full_name AS driver_name,
                COALESCE(t.total_cash, 0) AS total_cash_received,
                COALESCE(h.total_submitted, 0) AS total_cash_submitted
            FROM drivers d
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN (
                SELECT driver_id, SUM(cash_aed) AS total_cash
                FROM trips
                WHERE trip_date BETWEEN $1 AND $2 AND is_deleted = false
                GROUP BY driver_id
            ) t ON t.driver_id = d.id
            LEFT JOIN (
                SELECT driver_id, SUM(amount_aed) AS total_submitted
                FROM cash_handovers
                WHERE submitted_at::date BETWEEN $1 AND $2
                GROUP BY driver_id
            ) h ON h.driver_id = d.id
            WHERE t.driver_id IS NOT NULL OR h.driver_id IS NOT NULL
            ORDER BY p.full_name
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let received = r.total_cash_received.unwrap_or(Decimal::ZERO);
                let submitted = r.total_cash_submitted.unwrap_or(Decimal::ZERO);
                CashFlowRow {
                    driver_name: r.driver_name,
                    total_cash_received: received,
                    total_cash_submitted: submitted,
                    shortfall: received - submitted,
                }
            })
            .collect())
    }

    pub async fn leave_report(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<LeaveReportRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            driver_name: String,
            total_leave_days: Option<i64>,
            total_permissions: Option<i64>,
            pending_count: Option<i64>,
            approved_count: Option<i64>,
            rejected_count: Option<i64>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                p.full_name AS driver_name,
                COALESCE(SUM(CASE WHEN lr.type = 'leave'
                    THEN (lr.to_date - lr.from_date + 1) ELSE 0 END), 0) AS total_leave_days,
                COALESCE(SUM(CASE WHEN lr.type = 'permission' THEN 1 ELSE 0 END), 0) AS total_permissions,
                COALESCE(SUM(CASE WHEN lr.status = 'pending' THEN 1 ELSE 0 END), 0) AS pending_count,
                COALESCE(SUM(CASE WHEN lr.status = 'approved' THEN 1 ELSE 0 END), 0) AS approved_count,
                COALESCE(SUM(CASE WHEN lr.status = 'rejected' THEN 1 ELSE 0 END), 0) AS rejected_count
            FROM drivers d
            JOIN profiles p ON p.id = d.profile_id
            JOIN leave_requests lr ON lr.driver_id = d.id
            WHERE lr.from_date BETWEEN $1 AND $2
            GROUP BY p.full_name
            ORDER BY p.full_name
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| LeaveReportRow {
                driver_name: r.driver_name,
                total_leave_days: r.total_leave_days.unwrap_or(0),
                total_permissions: r.total_permissions.unwrap_or(0),
                pending_count: r.pending_count.unwrap_or(0),
                approved_count: r.approved_count.unwrap_or(0),
                rejected_count: r.rejected_count.unwrap_or(0),
            })
            .collect())
    }

    pub async fn salary_report(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<SalaryReportRow>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            driver_name: String,
            period: String,
            salary_type: String,
            gross: Decimal,
            deductions: Decimal,
            net_payable: Decimal,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                p.full_name AS driver_name,
                s.period_month::text AS period,
                s.salary_type_snapshot::text AS salary_type,
                s.base_amount_aed AS gross,
                s.advance_deduction_aed AS deductions,
                s.net_payable_aed AS net_payable
            FROM salaries s
            JOIN drivers d ON d.id = s.driver_id
            JOIN profiles p ON p.id = d.profile_id
            WHERE s.period_month BETWEEN $1 AND $2
            ORDER BY s.period_month DESC, p.full_name
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SalaryReportRow {
                driver_name: r.driver_name,
                period: r.period,
                salary_type: r.salary_type,
                gross: r.gross,
                deductions: r.deductions,
                net_payable: r.net_payable,
            })
            .collect())
    }
}
