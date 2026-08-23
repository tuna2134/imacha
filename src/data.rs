use sqlx::PgPool;

pub struct Data {
    pub pool: PgPool,
}

impl Data {
    pub async fn setup(database_uri: String) -> anyhow::Result<Self> {
        let pool = PgPool::connect(&database_uri).await?;
        Ok(Self { pool })
    }
}
