use serenity::{client::ClientBuilder, framework::StandardFramework};
use songbird::SerenityInit;

mod emoji;
mod groups;
mod handler;
mod help;

pub trait FrameworkBuilder<'a> {
    fn attach_framework(self) -> Self;
    fn attach_event_handler(self) -> Self;
}

pub trait AttachableClientBuilder<'a> {
    fn build(self) -> Self;
}

impl<'a> FrameworkBuilder<'a> for ClientBuilder<'a> {
    fn attach_framework(self) -> Self {
        let framework = StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .before(groups::hooks::before)
            .after(groups::hooks::after)
            .help(&help::HELP_COMMAND)
            .group(&groups::general::GENERAL_GROUP)
            .group(&groups::music::MUSIC_GROUP);

        self.framework(framework)
    }

    fn attach_event_handler(self) -> Self {
        let mut handler = handler::EventHandler::default();
        handler.register(Box::new(groups::general::Handler));
        handler.register(Box::new(groups::music::Handler));

        self.event_handler(handler)
    }
}

impl<'a> AttachableClientBuilder<'a> for ClientBuilder<'a> {
    fn build(self) -> Self {
        self.attach_framework()
            .attach_event_handler()
            .register_songbird()
    }
}
