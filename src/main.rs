#![warn(clippy::str_to_string)]

mod commands;

pub mod types;
pub mod secrets;

use std::ops::Deref;
use anyhow::{Error, Result};
use poise::{async_trait, serenity_prelude as serenity};
use shuttle_serenity::ShuttleSerenity;

use log::{debug, info};
use poise::serenity_prelude::EventHandler;
use shuttle_secrets::SecretStore;
use crate::secrets::SecretsUtils;

type Context<'a> = poise::Context<'a, types::Data, Error>;

struct Bot {}
#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: serenity::Context, ready: serenity::Ready){
        info!("{} is connected!", ready.user.name);
    }
}

async fn on_error(error: poise::FrameworkError<'_, types::Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command {error, ctx, ..} => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,)
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<()> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleSerenity{
    let token = SecretsUtils::get_secret("DISCORD_TOKEN", &secret_store).expect("DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::utilities::help(),
                commands::utilities::uptime()
            ],
            on_error: |error | {
                Box::pin(async move {
                    on_error(error).await
                })
            },
            pre_command: |ctx| {
                Box::pin(async move {
                    let channel_name = &ctx
                        .channel_id()
                        .name(&ctx)
                        .await
                        .unwrap_or_else(|_| "<unknown>".to_owned());
                    let author = &ctx.author().name;

                    info!(
                        "{} in {} used slash command '{}'",
                        author,
                        channel_name,
                        &ctx.invoked_command_name()
                    );
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    println!("Executed command {}!", ctx.command().qualified_name);
                })
            },
            ..Default::default()

        })
        .setup(move |ctx, ready, framework | {
            Box::pin(async move {
                let data = types::Data::new(&secret_store)?;
                debug!("Registering commands..");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                debug!("Setting activity text");
                ctx.set_activity(Some(serenity::ActivityData::listening("/help")));

                Ok(data)
            })
        })
        .build();


    let bot = Bot {};
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(bot)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
