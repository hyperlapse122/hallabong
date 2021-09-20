use serenity::framework::StandardFramework;

pub mod groups;
mod handler;

pub struct FrameworkManager;

impl FrameworkManager {
    pub fn framework() -> StandardFramework {
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&groups::general::GENERAL_GROUP)
    }

    pub fn handler() -> handler::EventHandler {
        let mut handler = handler::EventHandler::default();
        handler.register(Box::new(groups::general::Handler));
        handler
    }
}
