use std::env;

use captchas::ImageCaptcha;
use poise::{
    FrameworkContext,
    serenity_prelude::{
        self as serenity, CreateAttachment, CreateButton, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage, FullEvent, Interaction, Role,
    },
};

use crate::{
    data::Data,
    db::captcha::{get_role_id_by_guild_id, set_captcha},
};

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

async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    _frame: FrameworkContext<'_, Data, anyhow::Error>,
    data: &Data,
) -> anyhow::Result<()> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        FullEvent::InteractionCreate { interaction } => {
            if let Interaction::Component(component) = interaction {
                if component.data.custom_id == "captcha" {
                    component
                        .create_response(
                            &ctx.http,
                            serenity::CreateInteractionResponse::Defer(
                                CreateInteractionResponseMessage::new().ephemeral(true),
                            ),
                        )
                        .await?;
                    if let Some(role_id) =
                        get_role_id_by_guild_id(&data.pool, component.guild_id.unwrap().into())
                            .await?
                    {
                        let captcha = ImageCaptcha::default();
                        let img = captcha.generate("938223")?;
                        component
                            .create_followup(
                                &ctx.http,
                                CreateInteractionResponseFollowup::new()
                                    .add_file(CreateAttachment::bytes(img, "captcha.png"))
                                    .content("認証コードは以下の画像です。認証を押して、入力してください。"),
                            )
                            .await?;
                    }
                }
            }
        }
        _ => {}
    }
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
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
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
