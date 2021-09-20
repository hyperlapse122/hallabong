use std::env;

use serenity::client::ClientBuilder;
use songbird::SerenityInit;

mod framework;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = ClientBuilder::new(&token)
        .event_handler(framework::FrameworkManager::handler())
        .framework(framework::FrameworkManager::framework())
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
