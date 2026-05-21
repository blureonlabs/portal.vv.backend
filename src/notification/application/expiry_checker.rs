use sqlx::PgPool;
use std::sync::Arc;
use crate::notification::application::service::NotificationService;

pub async fn check_and_notify_expiring_documents(
    pool: &PgPool,
    notification: &Arc<NotificationService>,
) {
    #[derive(sqlx::FromRow)]
    struct ExpiryRow {
        file_name: String,
        doc_type: String,
        expiry_date: Option<chrono::NaiveDate>,
        email: String,
    }

    let rows = sqlx::query_as::<_, ExpiryRow>(
        r#"
        SELECT d.file_name, d.doc_type::text AS doc_type, d.expiry_date,
               COALESCE(p.email, '') AS email
        FROM documents d
        LEFT JOIN drivers dr ON dr.id = d.entity_id AND d.entity_type = 'driver'
        LEFT JOIN profiles p ON p.id = dr.profile_id
        WHERE d.expiry_date IS NOT NULL
          AND d.expiry_date <= CURRENT_DATE + INTERVAL '30 days'
          AND d.expiry_date >= CURRENT_DATE
        "#,
    )
    .fetch_all(pool)
    .await;

    if let Ok(rows) = rows {
        for row in rows {
            if !row.email.is_empty() {
                let _ = notification
                    .send_broadcast_email(
                        &row.email,
                        &format!("Document Expiring Soon: {}", row.file_name),
                        &format!(
                            "<p>Your {} document <strong>{}</strong> is expiring on <strong>{}</strong>. Please renew it as soon as possible.</p>",
                            row.doc_type,
                            row.file_name,
                            row.expiry_date.map(|d| d.to_string()).unwrap_or_default()
                        ),
                    )
                    .await;
            }
        }
    }
}
