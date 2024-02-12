/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use crate::db::DB;
use anyhow::Result;

pub trait DbEntry {
    async fn new(db: &DB, entry: Self) -> Result<Self>
    where
        Self: Sized;
    async fn get_by_id(db: &DB, id: i32) -> Result<Self>
    where
        Self: Sized;
}
