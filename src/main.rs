#![feature(in_band_lifetimes)]

use std::env;

use serenity::client::ClientBuilder;
use crate::framework::AttachableClientBuilder;

mod framework;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = ClientBuilder::new(&token)
        .attach_framework()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
