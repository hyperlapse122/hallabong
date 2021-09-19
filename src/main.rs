use std::env;

use serenity::{
    client::ClientBuilder,
    framework::{
        StandardFramework,
    },
};
use songbird::SerenityInit;

mod general;

use crate::general::{GENERAL_GROUP, Handler};

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let mut client = ClientBuilder::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
