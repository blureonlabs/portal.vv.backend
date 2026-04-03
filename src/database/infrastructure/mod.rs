use crate::database::domain::DatabasePool;

pub struct PgDatabase {
    pool: sqlx::PgPool,
}

impl PgDatabase {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(3)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

impl DatabasePool for PgDatabase {
    fn pg_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
