/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
pub mod types;
pub mod calendar;

use anyhow::{anyhow, Result};

pub struct DB {
    pub pool: sqlx::PgPool
}

impl DB {
    pub async fn migrate(&self) -> Result<()>
    {
       match sqlx::migrate!().run(&self.pool).await {
           Ok(_) => Ok(()),
           Err(e) => Err(anyhow!(format!("Failed to migrate: {}", e)))
       }
    }
}