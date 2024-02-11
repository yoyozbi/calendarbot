use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use anyhow::Result;
use crate::types::Data;


#[poise::command(prefix_command, slash_command, category = "Utilities", track_edits)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<()> {
    let extra_text_at_bottom = "\
    Type /help command for more info on a command.\
    You can edit your message to the bot and the bot will edit its response.";

    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom,
            ephemeral: true,
            ..Default::default()
        }
    ).await?;
    Ok(())
}
#[poise::command(prefix_command, slash_command, category = "Utilities", track_edits)]
pub async fn uptime(
    ctx: Context<'_>,
) -> Result<()> {
    let uptime = std::time::Instant::now() - ctx.data().bot_start_time;

    let div_mod = |a, b| (a / b, a % b);

    let seconds = uptime.as_secs();
    let (minutes, seconds) = div_mod(seconds, 60);
    let (hours, minutes) = div_mod(minutes, 60);
    let (days, hours) = div_mod(hours, 24);

    ctx.say(format!(
        "Uptime: {}d {}h {}m {}s",
        days, hours, minutes, seconds
    ))
        .await?;

    Ok(())
}