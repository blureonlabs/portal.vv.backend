use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::report::domain::entity::{
    DriverSummaryRow, FinanceReport, FinanceSummaryRow, TripDetailRow,
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
}
