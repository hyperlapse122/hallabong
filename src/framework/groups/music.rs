use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        Args,
        CommandResult, macros::{command, group},
    },
    http::Http,
    model::{channel::Message, prelude::ChannelId},
};
use serenity::model::channel::Channel;
use songbird::{
    create_player, Event, EventContext, EventHandler as VoiceEventHandler,
    input::restartable::Restartable, TrackEvent,
};
use songbird::driver::Bitrate;

use super::super::error::Error;

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
                .say(
                    &self.http,
                    &format!(
                        "Tracks ended: {}.",
                        track_list[0]
                            .1
                            .metadata()
                            .clone()
                            .title
                            .unwrap_or_else(|| "Unknown".to_string())
                    ),
                )
                .await
                .ok()?;
        }

        None
    }
}

struct ChannelDurationNotifier {
    channel_id: ChannelId,
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl VoiceEventHandler for ChannelDurationNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let count_before = self.count.fetch_add(1, Ordering::Relaxed);
        println!("I've been in {} for {} minutes!", self.channel_id.0, count_before + 1);

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
        self.channel_id
            .say(&self.http, "Song faded out completely!")
            .await
            .ok()?;

        None
    }
}

#[command]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;

    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        Err(Error::AlreadyDeafened)?;
    }

    handler
        .deafen(true)
        .await
        .map_err(|e| Error::Other(e.into()))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("j")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel = guild
        .voice_states
        .get(&msg.author.id)
        .ok_or(Error::NotInVoiceChannel)?;


    let channel_id = channel.channel_id.ok_or(Error::NotInVoiceChannel)?;
    let guild_channel = guild.channels.get(&channel_id).ok_or(Error::Unknown)?;
    let bitrate = match guild_channel {
        Channel::Guild(e) => { e.bitrate.ok_or(Error::NotInVoiceChannel) }
        _ => { Err(Error::NotInVoiceChannel) }
    }?;

    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let (handle_lock, success) = manager.join(guild_id, channel_id).await;

    success.map_err(|e| Error::Other(e.into()))?;

    let chan_id = msg.channel_id;

    let send_http = ctx.http.clone();

    let mut handle = handle_lock.lock().await;

    handle.set_bitrate(Bitrate::BitsPerSecond(bitrate as i32));
    handle.add_global_event(
        Event::Track(TrackEvent::End),
        TrackEndNotifier {
            channel_id: chan_id,
            http: send_http,
        },
    );

    handle.add_global_event(
        Event::Periodic(Duration::from_secs(60), None),
        ChannelDurationNotifier {
            channel_id: chan_id,
            count: Default::default(),
        },
    );

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("l")]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;

    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;

    manager
        .remove(guild_id)
        .await
        .map_err(|e| Error::Other(e.into()))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("m")]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;

    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        Err(Error::AlreadyMuted)?;
    }
    handler
        .mute(true)
        .await
        .map_err(|e| Error::Other(e.into()))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;
    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;
    let mut handler = handler_lock.lock().await;

    handler
        .deafen(false)
        .await
        .map_err(|e| Error::Other(e.into()))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;
    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;
    let mut handler = handler_lock.lock().await;

    handler
        .mute(false)
        .await
        .map_err(|e| Error::Other(e.into()))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[num_args(1)]
#[aliases("q")]
async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args
        .single::<String>()
        .map_err(|_| Error::InvalidArguments)?;

    if !url.starts_with("http") {
        Err(Error::InvalidArguments)?;
    }

    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;
    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;
    let mut handler = handler_lock.lock().await;

    // Here, we use lazy restartable sources to make sure that we don't pay
    // for decoding, playback on tracks which aren't actually live yet.
    let source = Restartable::ytdl(url, true)
        .await
        .map_err(|e| Error::Other(e.into()))?;

    let (mut track, _) = create_player(source.into());

    track.set_volume(0.5);
    handler.enqueue(track);

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;
    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;
    let handler = handler_lock.lock().await;

    let queue = handler.queue();
    queue.skip().map_err(|e| Error::Other(e.into()))?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("s")]
async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;
    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;
    let handler = handler_lock.lock().await;

    let queue = handler.queue();
    queue.stop();

    Ok(())
}

#[command]
#[only_in(guilds)]
#[num_args(1)]
async fn seek(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let time = args.single::<u64>().map_err(|_| Error::InvalidArguments)?;

    let guild_id = msg.guild(&ctx.cache).ok_or(Error::Unknown)?.id;
    let manager = songbird::get(ctx)
        .await
        .ok_or(Error::SongbirdInitialization)?
        .clone();

    let handler_lock = manager.get(guild_id).ok_or(Error::NotInVoiceChannel)?;
    let handler = handler_lock.lock().await;

    let queue = handler.queue();

    let track_handle = queue.current().ok_or(Error::Unknown)?;

    if track_handle.is_seekable() {
        track_handle
            .seek_time(Duration::from_secs(time))
            .map_err(|e| Error::Other(e.into()))?;
    } else {
        Err(Error::NotSeekable)?;
    }

    Ok(())
}
