/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use crate::db::types::DbEntry;
use crate::db::DB;
use anyhow::Result;
use chrono_tz::Tz;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct RawCalendar {
    pub id: Option<i32>,
    #[allow(non_snake_case)]
    pub googleid: Option<String>,
    pub timezone: Option<String>,
    #[allow(non_snake_case)]
    pub pollinterval: Option<i32>,
}

pub struct Calendar {
    pub id: Option<i32>,
    pub google_id: String,
    pub timezone: Tz,
    pub poll_interval: i32,
}

impl DbEntry for Calendar {
    async fn new(db: &DB, entry: Self) -> Result<Calendar> {
        let query = sqlx::query_as!(
            RawCalendar,
            r#"INSERT INTO calendars
            (googleId, timezone, pollInterval)
            VALUES ($1, $2, $3)
            RETURNING id, googleId, timezone, pollInterval"#,
            entry.google_id,
            entry.timezone.to_string(),
            entry.poll_interval
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(Calendar {
            id: query.id,
            google_id: query.googleid.expect("No googleId"),
            timezone: query.timezone.expect("No timezone").parse().unwrap(),
            poll_interval: query.pollinterval.expect("No poll_interval"),
        })
    }

    async fn get_by_id(db: &DB, id: i32) -> Result<Self> {
        let query = sqlx::query_as!(
            RawCalendar,
            r#"SELECT id, googleId, timezone, pollInterval
            FROM Calendars WHERE id = $1"#,
            id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(Calendar {
            id: query.id,
            google_id: query.googleid.expect("No googleId"),
            timezone: query.timezone.expect("No timezone").parse().unwrap(),
            poll_interval: query.pollinterval.expect("No poll_interval"),
        })
    }
}
