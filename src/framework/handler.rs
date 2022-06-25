use serenity::{
    async_trait,
    client::{Context, EventHandler as EventHandlerBase},
    model::gateway::Ready,
};
use serenity::model::channel::Reaction;

#[derive(Default)]
pub struct EventHandler {
    handlers: Vec<Box<dyn EventHandlerBase>>,
}

impl EventHandler {
    pub fn register(&mut self, event_handler: Box<dyn EventHandlerBase>) {
        self.handlers.push(event_handler);
    }
}

#[async_trait]
impl EventHandlerBase for EventHandler {
    async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {
        for handler in self.handlers.iter() {
            handler.reaction_add(_ctx.clone(), _add_reaction.clone()).await;
        }
    }

    async fn ready(&self, c: Context, r: Ready) {
        for handler in self.handlers.iter() {
            handler.ready(c.clone(), r.clone()).await;
        }
    }
}
