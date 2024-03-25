use crate::commands;
use crate::types;
use anyhow::Error;
use log::{debug, info};
use poise::serenity_prelude as serenity;

async fn on_error(error: poise::FrameworkError<'_, types::Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,)
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

pub struct Discord {
    token: String,
    data: types::Data,
    intents: serenity::GatewayIntents,
}

impl Discord {
    pub fn new(
        token: String,
        data: types::Data,
        gateway_intents: serenity::GatewayIntents,
    ) -> Self {
        Self {
            token,
            data,
            intents: gateway_intents,
        }
    }

    pub async fn init(self) -> serenity::Client {
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![
                    commands::utilities::help(),
                    commands::utilities::uptime(),
                    commands::utilities::age(),
                ],
                on_error: |error| Box::pin(async move { on_error(error).await }),
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
                        debug!(
                            "{} executed command \"{}\"",
                            ctx.author().tag(),
                            ctx.command().qualified_name
                        );
                    })
                },
                ..Default::default()
            })
            .setup(move |ctx, ready, framework| {
                Box::pin(async move {
                    debug!("Registering commands..");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    debug!("Setting activity text...");
                    ctx.set_activity(Some(serenity::ActivityData::listening("/help")));

                    info!("{} is ready !", ready.user.name);

                    Ok(self.data)
                })
            })
            .build();

        serenity::Client::builder(self.token, self.intents)
            .framework(framework)
            .await
            .expect("Failed to create client")
    }
}
