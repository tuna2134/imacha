use sqlx::PgPool;

pub struct Data {
    pub pool: PgPool,
}

impl Data {
    pub async fn setup(database_uri: String) -> anyhow::Result<Self> {
        let pool = PgPool::connect(&database_uri).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
}
