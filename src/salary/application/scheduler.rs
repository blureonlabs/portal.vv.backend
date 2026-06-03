use chrono::{Local, Datelike};
use sqlx::PgPool;
use std::sync::Arc;
use crate::settings::domain::repository::SettingsRepository;

pub async fn check_and_generate_salaries(
    pool: &PgPool,
    settings_repo: &Arc<dyn SettingsRepository>,
) {
    // 1. Check if auto-generate is enabled
    let settings = match settings_repo.list().await {
        Ok(s) => s,
        Err(e) => { tracing::error!("Failed to load settings for auto-salary: {}", e); return; }
    };

    let enabled = settings.iter()
        .find(|s| s.key == "salary_auto_generate_enabled")
        .map(|s| s.value == "true")
        .unwrap_or(false);

    if !enabled {
        tracing::debug!("Auto-salary generation is disabled");
        return;
    }

    // 2. Check if today is the configured day
    let target_day: u32 = settings.iter()
        .find(|s| s.key == "salary_auto_generate_day")
        .and_then(|s| s.value.parse().ok())
        .unwrap_or(25);

    let today = Local::now().date_naive();
    if today.day() != target_day {
        tracing::debug!("Auto-salary: today is day {}, target is day {}", today.day(), target_day);
        return;
    }

    tracing::info!("Auto-salary generation triggered for day {}", target_day);

    // 3. Get all active drivers
    let drivers = match sqlx::query_as::<_, (uuid::Uuid, String)>(
        "SELECT d.id, p.full_name FROM drivers d JOIN profiles p ON p.id = d.profile_id WHERE d.is_active = true"
    )
    .fetch_all(pool)
    .await {
        Ok(d) => d,
        Err(e) => { tracing::error!("Failed to fetch drivers for auto-salary: {}", e); return; }
    };

    tracing::info!("Auto-salary: generating for {} active drivers", drivers.len());

    // 4. For each driver, log that auto-generation was triggered
    // Note: actual salary generation requires trip data inputs from the frontend.
    // Auto-salary here just logs the trigger; a separate workflow collects earnings.
    for (driver_id, name) in &drivers {
        tracing::info!("Auto-salary: would generate salary for {} ({})", name, driver_id);
    }

    // Full auto-generation with trip aggregation would require:
    // 1. Fetch monthly earnings from trips
    // 2. Build GenerateRequest with all inputs
    // 3. Call salary_svc.generate()
    // This is deferred — requires defining which inputs are auto-populated vs manual
}
