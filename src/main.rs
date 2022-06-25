mod framework;

use std::env;

use framework::AttachableClientBuilder;
use serenity::client::ClientBuilder;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = ClientBuilder::new(&token)
        .build()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
