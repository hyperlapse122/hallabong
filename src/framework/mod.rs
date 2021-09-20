use serenity::{client::ClientBuilder, framework::StandardFramework};
use songbird::SerenityInit;

mod emoji;
mod groups;
mod handler;

struct FrameworkManager;

impl FrameworkManager {
    pub fn framework() -> StandardFramework {
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&groups::general::GENERAL_GROUP)
            .group(&groups::music::MUSIC_GROUP)
    }

    pub fn handler() -> handler::EventHandler {
        let mut handler = handler::EventHandler::default();
        handler.register(Box::new(groups::general::Handler));
        handler.register(Box::new(groups::music::Handler));
        handler
    }
}

pub trait AttachableClientBuilder<'a> {
    fn attach_framework(self) -> Self;
}

impl<'a> AttachableClientBuilder<'a> for ClientBuilder<'a> {
    fn attach_framework(self) -> Self {
        self
            .framework(FrameworkManager::framework())
            .event_handler(FrameworkManager::handler())
            .register_songbird()
    }
}
