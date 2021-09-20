use serenity::{async_trait, client::{EventHandler as EventHandlerBase, Context}, model::gateway::Ready};

pub struct EventHandler {
    handlers: Vec<Box<dyn EventHandlerBase>>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl EventHandler {
    pub fn register(&mut self, event_handler: Box<dyn EventHandlerBase>) {
        self.handlers.push(event_handler);
    }
}

#[async_trait]
impl EventHandlerBase for EventHandler {
    async fn ready(&self, c: Context, r: Ready) {
        for handler in self.handlers.iter() {
            handler.ready(c.clone(), r.clone()).await;
        }
    }
}
