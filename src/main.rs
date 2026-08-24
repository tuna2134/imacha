use std::{env, time::Duration};

use captchas::ImageCaptcha;
use poise::{
    FrameworkContext, Modal,
    serenity_prelude::{
        self as serenity, CreateAttachment, CreateButton, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage, FullEvent, Interaction, Role, RoleId,
    },
};

use crate::{
    data::Data,
    db::captcha::{get_role_id_by_guild_id, set_captcha},
};

mod data;
mod db;

type Context<'a> = poise::Context<'a, data::Data, anyhow::Error>;

struct ContextRef<'a>(&'a serenity::Context);

impl AsRef<serenity::Context> for ContextRef<'_> {
    fn as_ref(&self) -> &serenity::Context {
        self.0
    }
}

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

#[derive(Debug, Modal)]
struct CaptchaModal {
    #[name = "認証コード"]
    #[placeholder = "認証コードを入力してください。"]
    #[min_length = 6]
    code: String,
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
            if let Interaction::Component(component) = interaction
                && component.data.custom_id == "captcha"
            {
                component
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::Defer(
                            CreateInteractionResponseMessage::new().ephemeral(true),
                        ),
                    )
                    .await?;
                if let Some(role_id) =
                    get_role_id_by_guild_id(&data.pool, component.guild_id.unwrap().into()).await?
                {
                    let captcha = ImageCaptcha::default();
                    let text: i32 = rand::random_range(100000..=999999);
                    let img = captcha.generate(&text.to_string())?;
                    let custom_id = format!("captcha:{}", interaction.id());
                    let msg = component
                        .create_followup(
                            &ctx.http,
                            CreateInteractionResponseFollowup::new()
                                .add_file(CreateAttachment::bytes(img, "captcha.png"))
                                .content(
                                    "認証コードは以下の画像です。認証を押して、入力してください。",
                                )
                                .button(CreateButton::new(custom_id.clone()).label("認証する。")),
                        )
                        .await?;
                    let interaction = serenity::collector::ComponentInteractionCollector::new(ctx)
                        .timeout(Duration::from_secs(60))
                        .filter(move |interaction| interaction.data.custom_id == custom_id)
                        .await;
                    if let Some(interaction) = interaction {
                        let result = poise::modal::execute_modal_on_component_interaction::<
                            CaptchaModal,
                        >(
                            ContextRef(ctx), interaction, None, None
                        )
                        .await?;
                        if result.unwrap().code == text.to_string() {
                            ctx.http
                                .add_member_role(
                                    component.guild_id.unwrap(),
                                    component.user.id,
                                    RoleId::from(role_id as u64),
                                    Some("認証に成功しました。"),
                                )
                                .await?;
                            println!("認証に成功したため、ロールをユーザーに渡しました。");
                            component
                                .edit_followup(
                                    &ctx.http,
                                    msg.id,
                                    CreateInteractionResponseFollowup::new()
                                        .content("認証完了しました。")
                                        .components(Vec::new()),
                                )
                                .await?;
                        } else {
                            component
                                .edit_followup(
                                    &ctx.http,
                                    msg.id,
                                    CreateInteractionResponseFollowup::new()
                                        .content("認証コードが間違っています。再度認証をやり直してください。")
                                        .components(Vec::new()),
                                )
                                .await?;
                        }
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
