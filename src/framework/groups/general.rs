use serenity::{
    async_trait,
    client::{
        Context,
        EventHandler,
    },
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::{
        channel::{Message, ReactionType},
        gateway::Ready,
    },
};
use crate::framework::emoji;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping)]
pub struct General;


#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.react(&ctx.http, ReactionType::Unicode(emoji::SUCCESS.to_string())).await?;
    Ok(())
}
