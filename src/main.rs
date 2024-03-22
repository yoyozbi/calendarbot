/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
mod commands;
pub mod models;
pub mod schema;
pub mod types;

use anyhow::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use poise::serenity_prelude as serenity;
use std::env;

use dotenvy::dotenv;
use log::{debug, info};

type Context<'a> = poise::Context<'a, types::Data, Error>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

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

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::non_privileged();

    let data = types::Data::new(connection).expect("Unable to load config!");

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

                debug!("Setting activity text");
                ctx.set_activity(Some(serenity::ActivityData::listening("/help")));

                info!("{} is ready !", ready.user.name);

                Ok(data)
            })
        })
        .build();

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
}
