use std::env;

use poise::serenity_prelude::{self as serenity, CreateButton, CreateMessage, Role};

use crate::{data::Data, db::captcha::set_captcha};

mod data;
mod db;

type Context<'a> = poise::Context<'a, data::Data, anyhow::Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), anyhow::Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn captcha(ctx: Context<'_>, role: Role) -> anyhow::Result<()> {
    set_captcha(
        &ctx.data().pool,
        ctx.guild_id().unwrap().into(),
        role.id.into(),
    )
    .await?;
    let msg = CreateMessage::new()
        .button(CreateButton::new("captcha").label("認証"))
        .content("認証始めるにはボタンを押してください");
    ctx.channel_id().send_message(ctx.http(), msg).await?;
    ctx.say("Ok").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();
    let database_url = env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let data = Data::setup(database_url).await?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), captcha()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await?;
    Ok(())
}
