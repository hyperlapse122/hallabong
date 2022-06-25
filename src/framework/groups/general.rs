use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        CommandResult,
        macros::{command, group},
    },
    model::{channel::Message, gateway::Ready},
};
use serenity::framework::standard::Args;

use super::super::error::Error;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping, echo)]
pub struct General;

#[command]
async fn ping(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn echo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let text = args.single::<String>().map_err(|_| Error::InvalidArguments)?;

    msg.reply(&ctx.http, text).await?;

    Ok(())
}
