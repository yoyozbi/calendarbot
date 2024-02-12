/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use anyhow::{Error, Result};
use poise::serenity_prelude as serenity;

use shuttle_secrets::SecretStore;

use crate::secrets::SecretsUtils;

#[derive(Debug)]
pub struct Data {
    pub application_id: serenity::UserId,
    pub client_id: serenity::UserId,
    pub bot_start_time: std::time::Instant,
}

impl Data {
    pub fn new(secret_store: &SecretStore) -> Result<Data> {
        Ok(Self {
            application_id: SecretsUtils::get_secret("APPLICATION_ID", secret_store)
                .expect("APPLICATION_ID not found")
                .parse::<u64>()?
                .into(),
            client_id: SecretsUtils::get_secret("CLIENT_ID", secret_store)
                .expect("CLIENT_ID not found")
                .parse::<u64>()?
                .into(),
            bot_start_time: std::time::Instant::now()
        })
    }
}

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const EMBED_COLOR: (u8, u8, u8) = (0xb7, 0x47, 0x00);