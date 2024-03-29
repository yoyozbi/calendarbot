// @generated automatically by Diesel CLI.

diesel::table! {
    calendars (id) {
        id -> Int4,
        #[max_length = 90]
        googleid -> Varchar,
        #[max_length = 30]
        timezone -> Nullable<Varchar>,
        pollinterval -> Nullable<Int4>,
    }
}

diesel::table! {
    guilds (id) {
        id -> Int4,
        #[max_length = 64]
        discordid -> Varchar,
    }
}

diesel::table! {
    guilds_calendars (guild_id, calendar_id, channelid) {
        guild_id -> Int4,
        calendar_id -> Int4,
        #[max_length = 64]
        channelid -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(calendars, guilds, guilds_calendars,);
