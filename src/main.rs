use daphuulbot::feed;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::{gateway::Ready, prelude::ChannelId},
};
use std::{collections::HashSet, env};

/// EventHandler for signaling when the bot is connected and ready.
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Please set DISCORD_TOKEN as an env var.");
    let postable_channels = env::var("DISCORD_POSTABLE_CHANNELS")
        .expect("Please set allowed DISCORD_POSTABLE_CHANNELS")
        .split(",")
        .map(|str| Ok(ChannelId(str.parse::<u64>()?)))
        .collect::<anyhow::Result<HashSet<ChannelId>>>()
        .expect("Channel IDs must be u64s");

    let framework = StandardFramework::new().normal_message(feed::normal_message_hook);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client.");
    {
        let mut data = client.data.write().await;
        data.insert::<feed::PostableChannels>(postable_channels);
    }

    if let Err(err) = client.start().await {
        eprintln!("Client error: {:?}", err);
    }
}
