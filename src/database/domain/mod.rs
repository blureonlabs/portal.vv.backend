use async_trait::async_trait;


/// Abstraction over the database connection pool.
/// Infrastructure layer holds the concrete PgPool — nothing else needs to know.
#[async_trait]
pub trait DatabasePool: Send + Sync {
    fn pg_pool(&self) -> &sqlx::PgPool;
}
