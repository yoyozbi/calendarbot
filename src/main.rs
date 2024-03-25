/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
mod commands;
pub mod discord;
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

type Context<'a> = poise::Context<'a, types::Data, Error>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

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

    let mut client = discord::Discord::new(token, data, intents).init().await;

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
}
