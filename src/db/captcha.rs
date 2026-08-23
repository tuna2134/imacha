use sqlx::PgPool;

pub async fn set_captcha(pool: &PgPool, guild_id: i64, role_id: i64) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO captcha (guild_id, role_id) VALUES ($1, $2)
        ON CONFLICT(guild_id)
        DO UPDATE SET role_id=$2
        "#,
    )
    .bind(guild_id)
    .bind(role_id)
    .execute(pool)
    .await?;
    Ok(())
}
