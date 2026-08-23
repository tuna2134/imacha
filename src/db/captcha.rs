use sqlx::PgPool;

pub async fn set_captcha(pool: &PgPool, guild_id: i64, role_id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO captcha (guild_id, role_id) VALUES ($1, $2)
        ON CONFLICT(guild_id)
        DO UPDATE SET role_id = $2
        "#,
        guild_id,
        role_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_role_id_by_guild_id(pool: &PgPool, guild_id: i64) -> anyhow::Result<Option<i64>> {
    let rec = sqlx::query!(
        r#"
        SELECT role_id FROM captcha WHERE guild_id = $1
        "#,
        guild_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(rec.map(|r| r.role_id))
}
