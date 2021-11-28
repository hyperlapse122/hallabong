use std::collections::HashMap;
use std::sync::Arc;

use google_translate3::api::Translate as TranslateApi;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        Args,
        CommandResult, macros::{command, group},
    },
    model::channel::Message,
};
use songbird::typemap::TypeMapKey;
use tokio::sync::RwLock;
use translate3::api::TranslateTextRequest;

use super::super::error::Error;

pub struct LastTranslationLanguageCache;

impl TypeMapKey for LastTranslationLanguageCache {
    type Value = Arc<RwLock<HashMap<u64, String>>>;
}

pub struct GoogleTranslate;

impl TypeMapKey for GoogleTranslate {
    type Value = Arc<RwLock<TranslateApi>>;
}

pub struct GoogleProjectId;

impl TypeMapKey for GoogleProjectId {
    type Value = Arc<String>;
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[group]
#[commands(translate)]
pub struct Translate;

#[command]
#[aliases("t")]
async fn translate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let reference_message = msg.referenced_message.clone()
        .ok_or(Error::DetailedInvalidArguments("Reference message to translate".into()))?.content.clone();

    let data = ctx.data.write().await;

    let last_translation_language_cache_lock = data.get::<LastTranslationLanguageCache>().ok_or(Error::Unknown)?.clone();
    let mut last_translation_language_cache = last_translation_language_cache_lock.write().await;

    let translate_lock = data.get::<GoogleTranslate>().ok_or(Error::Unknown)?.clone();
    let translate = translate_lock.read().await;

    let parent = data.get::<GoogleProjectId>().ok_or(Error::Unknown)?.clone();

    let parent = format!("projects/{}", parent);

    let target_language = args.single::<String>();

    let target_language = match (last_translation_language_cache.get(&msg.author.id.0), target_language) {
        (_, Ok(r)) => { Ok(r) }
        (Some(r), _) => { Ok(r.clone()) }
        (None, Err(_)) => { Err(Error::InvalidArguments) }
    }?;

    let translate_response = translate.projects().translate_text(TranslateTextRequest {
        contents: Some(vec![reference_message]),
        glossary_config: None,
        labels: None,
        mime_type: None,
        model: None,
        source_language_code: None,
        target_language_code: Some(target_language.clone()),
    }, &parent).doit().await
        .map_err(|e| Error::Other(e.into()))?.1
        .translations.ok_or(Error::Unknown)?[0].clone()
        .translated_text.ok_or(Error::Unknown)?;

    let translate_response = html_escape::decode_html_entities(&translate_response);

    msg.reply_ping(&ctx.http, translate_response).await?;

    last_translation_language_cache.insert(msg.author.id.0, target_language);

    Ok(())
}
