use serenity::model::channel::{Message, ReactionType, Reaction, MessageReaction};
use serenity::client::Context;
use serenity::Result as SerenityResult;

pub async fn work_before(ctx: &Context, msg: &Message) -> SerenityResult<Reaction> {
    msg.react(&ctx.http, ReactionType::Unicode(super::STOPWATCH.to_string())).await
}

pub async fn work_finished(ctx: &Context, msg: &Message) -> SerenityResult<()> {
    msg.delete_reaction_emoji(&ctx.http, ReactionType::Unicode(super::STOPWATCH.to_string())).await
}

pub async fn success(ctx: &Context, msg: &Message) -> SerenityResult<Reaction> {
    msg.react(&ctx.http, ReactionType::Unicode(super::SUCCESS.to_string())).await
}

pub async fn failed(ctx: &Context, msg: &Message) -> SerenityResult<Reaction> {
    msg.react(&ctx.http, ReactionType::Unicode(super::FAILED.to_string())).await
}
