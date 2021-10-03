use serenity::framework::standard::Args;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::{channel::Message, gateway::Ready},
};
use thiserror::Error as ThisError;

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

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Arguments is wrong")]
    Arguments,
}

#[command]
async fn ping(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn echo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let text = args.single::<String>().map_err(|_| Error::Arguments)?;

    msg.reply(&ctx.http, text).await?;

    Ok(())
}
