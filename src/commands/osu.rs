use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn osu(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}