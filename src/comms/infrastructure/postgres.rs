use uuid::Uuid;
use crate::common::error::AppError;
use crate::comms::domain::entity::*;

pub struct PgCommsRepository {
    pool: sqlx::PgPool,
}

impl PgCommsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self { Self { pool } }

    pub async fn list(&self) -> Result<Vec<Broadcast>, AppError> {
        let rows = sqlx::query_as!(
            Broadcast,
            r#"SELECT b.id, b.subject, b.body,
                      b.channel AS "channel: BroadcastChannel",
                      b.target AS "target: BroadcastTarget",
                      b.target_driver_ids,
                      b.sent_by,
                      p.full_name AS sent_by_name,
                      b.recipient_count,
                      b.status AS "status: BroadcastStatus",
                      b.created_at
               FROM broadcasts b
               JOIN profiles p ON p.id = b.sent_by
               ORDER BY b.created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn create(
        &self,
        subject: &str,
        body: &str,
        channel: &BroadcastChannel,
        target: &BroadcastTarget,
        target_driver_ids: Option<&[Uuid]>,
        sent_by: Uuid,
    ) -> Result<Broadcast, AppError> {
        let row = sqlx::query_as!(
            Broadcast,
            r#"WITH ins AS (
                INSERT INTO broadcasts (subject, body, channel, target, target_driver_ids, sent_by)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
            )
            SELECT ins.id, ins.subject, ins.body,
                   ins.channel AS "channel: BroadcastChannel",
                   ins.target AS "target: BroadcastTarget",
                   ins.target_driver_ids,
                   ins.sent_by,
                   p.full_name AS sent_by_name,
                   ins.recipient_count,
                   ins.status AS "status: BroadcastStatus",
                   ins.created_at
            FROM ins
            JOIN profiles p ON p.id = ins.sent_by"#,
            subject, body,
            channel as &BroadcastChannel,
            target as &BroadcastTarget,
            target_driver_ids,
            sent_by
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_status(&self, id: Uuid, status: &BroadcastStatus, recipient_count: i32) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE broadcasts SET status = $2, recipient_count = $3 WHERE id = $1",
            id, status as &BroadcastStatus, recipient_count
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_driver_emails(&self, driver_ids: Option<&[Uuid]>) -> Result<Vec<(String, String)>, AppError> {
        // Returns (full_name, email) for target drivers
        let rows: Vec<(String, String)> = match driver_ids {
            Some(ids) => {
                sqlx::query_as(
                    "SELECT p.full_name, p.email FROM drivers d JOIN profiles p ON p.id = d.profile_id WHERE d.id = ANY($1) AND d.is_active = true"
                )
                .bind(ids)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    "SELECT p.full_name, p.email FROM drivers d JOIN profiles p ON p.id = d.profile_id WHERE d.is_active = true"
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows)
    }
}
