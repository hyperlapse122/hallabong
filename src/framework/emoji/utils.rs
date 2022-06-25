use serenity::client::Context;
use serenity::model::channel::{Message, Reaction, ReactionType};
use serenity::Result as SerenityResult;
use crate::framework::emoji::flags;

pub async fn work_before(ctx: &Context, msg: &Message) -> SerenityResult<Reaction> {
    msg.react(
        &ctx.http,
        ReactionType::Unicode(super::STOPWATCH.to_string()),
    )
        .await
}

pub async fn work_finished(ctx: &Context, msg: &Message) -> SerenityResult<()> {
    msg.delete_reaction_emoji(
        &ctx.http,
        ReactionType::Unicode(super::STOPWATCH.to_string()),
    )
        .await
}

pub async fn success(ctx: &Context, msg: &Message) -> SerenityResult<Reaction> {
    msg.react(&ctx.http, ReactionType::Unicode(super::SUCCESS.to_string()))
        .await
}

pub async fn failed(ctx: &Context, msg: &Message) -> SerenityResult<Reaction> {
    msg.react(&ctx.http, ReactionType::Unicode(super::FAILED.to_string()))
        .await
}

pub fn get_locale_by_flag(flag: &str) -> Option<&'static str> {
    match flag {
        flags::KO_KR => Some("ko-kr"),
        flags::JA_JP => Some("ja-jp"),
        flags::EN_US => Some("en-us"),
        _ => None
    }
}
