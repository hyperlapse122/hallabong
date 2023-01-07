use serenity::{client::ClientBuilder, framework::StandardFramework};
use songbird::SerenityInit;

mod emoji;
pub(crate) mod groups;
mod handler;
mod help;
mod error;

pub trait FrameworkBuilder<'a> {
    fn attach_framework(self) -> Self;
    fn attach_event_handler(self) -> Self;
}

pub trait AttachableClientBuilder<'a> {
    fn build(self) -> Self;
}

impl FrameworkBuilder<'_> for ClientBuilder {
    fn attach_framework(self) -> Self {
        let framework = StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .before(groups::hooks::before)
            .after(groups::hooks::after)
            .help(&help::HELP_COMMAND)
            .group(&groups::general::GENERAL_GROUP)
            .group(&groups::music::MUSIC_GROUP);

        #[cfg(feature = "translate")]
        let framework = framework.group(&groups::translate::TRANSLATE_GROUP);

        self.framework(framework)
    }

    fn attach_event_handler(self) -> Self {
        let mut handler = handler::EventHandler::default();
        handler.register(Box::new(groups::general::Handler));
        handler.register(Box::new(groups::music::Handler));

        #[cfg(feature = "translate")]
        handler.register(Box::new(groups::translate::Handler));

        self.event_handler(handler)
    }
}

impl AttachableClientBuilder<'_> for ClientBuilder {
    fn build(self) -> Self {
        self.attach_framework()
            .attach_event_handler()
            .register_songbird()
    }
}
