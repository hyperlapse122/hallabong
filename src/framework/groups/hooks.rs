use serenity::{client::Context, framework::standard::{macros::hook, CommandResult}, model::channel::Message};
use crate::framework::emoji::utils as emoji;

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    match emoji::work_before(ctx, msg).await {
        Ok(_) => true,
        Err(e) => {
            msg.reply_ping(&ctx.http, format!("Emoji Reaction Failed. Solve it and try again. The problem was:\n```{}```", e.to_string())).await.ok();
            false
        }
    }
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match emoji::work_finished(ctx, msg).await {
        Ok(_) => {}
        Err(e) => {
            msg.reply_ping(&ctx.http, format!("Emoji Reaction Remove Failed. The problem was:\n```{}```", e.to_string())).await.ok();
        }
    };

    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    };
}