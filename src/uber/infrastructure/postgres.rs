use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::{error::AppError, ports::{ExternalTrip, TripSourcePort}};

/// No-op stub — Uber API integration not yet contracted.
/// Satisfies the TripSourcePort so the trip module can be extended later.
pub struct UberTripSource;

#[async_trait]
impl TripSourcePort for UberTripSource {
    async fn fetch_trips(
        &self,
        _driver_id: Uuid,
        _from: NaiveDate,
        _to: NaiveDate,
    ) -> Result<Vec<ExternalTrip>, AppError> {
        Ok(vec![])
    }
}
