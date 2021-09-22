use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use serenity::{
    async_trait,
    client::{
        Context,
        EventHandler,
    },
    framework::{
        standard::{
            macros::{command, group},
            Args,
            CommandResult,
        },
    },
    http::Http,
    model::{
        channel::Message,
        prelude::ChannelId,
    },
};

use songbird::{input::restartable::Restartable, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent, create_player};
use crate::framework::emoji::utils as emoji;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[group]
#[commands(queue, skip, seek, stop, deafen, join, leave, mute, undeafen, unmute)]
pub struct Music;

struct TrackEndNotifier {
    channel_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            self.channel_id
                .say(&self.http, &format!("Tracks ended: {}.", track_list[0].1.metadata().clone().title.unwrap_or("Unknown".to_string())))
                .await.ok()?;
        }

        None
    }
}

struct ChannelDurationNotifier {
    channel_id: ChannelId,
    count: Arc<AtomicUsize>,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for ChannelDurationNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let count_before = self.count.fetch_add(1, Ordering::Relaxed);
        self.channel_id.say(&self.http, &format!("I've been in this channel for {} minutes!", count_before + 1)).await.ok()?;

        None
    }
}

struct SongEndNotifier {
    channel_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        self.channel_id.say(&self.http, "Song faded out completely!").await.ok()?;

        None
    }
}

#[command]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        msg.channel_id.say(&ctx.http, "Already deafened").await?;
    } else {
        if let Err(e) = handler.deafen(true).await {
            msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await?;
        } else {
            msg.channel_id.say(&ctx.http, "Deafened").await?;
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("j")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            emoji::success(ctx, msg).await?;
            msg.reply_ping(&ctx.http, "Not in a voice channel.").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if let Ok(_channel) = success {
        emoji::success(ctx, msg).await?;

        let chan_id = msg.channel_id;

        let send_http = ctx.http.clone();

        let mut handle = handle_lock.lock().await;

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                channel_id: chan_id,
                http: send_http,
            },
        );

        let send_http = ctx.http.clone();

        handle.add_global_event(
            Event::Periodic(Duration::from_secs(60), None),
            ChannelDurationNotifier {
                channel_id: chan_id,
                count: Default::default(),
                http: send_http,
            },
        );
    } else {
        emoji::failed(ctx, msg).await?;
        msg.reply_ping(&ctx.http, "Error joining the channel").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("l")]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await?;
        }

        emoji::success(ctx, msg).await?;
    } else {
        emoji::failed(ctx, msg).await?;
        msg.reply_ping(&ctx.http, "Not in a voice channel to play in").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("m")]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        msg.channel_id.say(&ctx.http, "Already muted").await?;
    } else {
        if let Err(e) = handler.mute(true).await {
            msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await?;
        }

        msg.channel_id.say(&ctx.http, "Now muted").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await?;
        }

        msg.channel_id.say(&ctx.http, "Undeafened").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel to undeafen in").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await?;
        }

        msg.channel_id.say(&ctx.http, "Unmuted").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel to unmute in").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[num_args(1)]
#[aliases("q")]
async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await?;

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        msg.channel_id.say(&ctx.http, "Must provide a valid URL").await?;

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await?;

                return Ok(());
            }
        };

        let (mut track, _) = create_player(source.into());

        track.set_volume(0.25);
        handler.enqueue(track);

        emoji::success(ctx, msg).await?;
    } else {
        emoji::failed(ctx, msg).await?;
        msg.reply_ping(&ctx.http, "Not in a voice channel to play in").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        emoji::success(ctx, msg).await?;
    } else {
        emoji::failed(ctx, msg).await?;
        msg.reply_ping(&ctx.http, "Not in a voice channel to play in").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("s")]
async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.stop();

        emoji::success(ctx, msg).await?;
    } else {
        emoji::failed(ctx, msg).await?;
        msg.reply_ping(&ctx.http, "Not in a voice channel to play in").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[num_args(1)]
async fn seek(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let time = match args.single::<u64>() {
        Ok(time) => time,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Must provide seek time by seconds").await?;

            return Ok(());
        }
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(track_handle) = queue.current() {
            if track_handle.is_seekable() {
                match track_handle.seek_time(Duration::from_secs(time)) {
                    Ok(_) => {
                        emoji::success(ctx, msg).await?;
                    }
                    Err(why) => {
                        println!("Track Seek Failed: {}", why.to_string());
                        emoji::failed(ctx, msg).await?;
                    }
                };
            } else {
                emoji::failed(ctx, msg).await?;
                msg.reply_ping(&ctx.http, format!("{} is not seekable.", track_handle.metadata().title.clone().unwrap_or("Content".to_string()))).await?;
            }
        }
    } else {
        msg.reply_ping(&ctx.http, format!("Not in a voice channel to play in")).await?;
    }

    Ok(())
}