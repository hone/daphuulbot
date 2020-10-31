use lazy_static::lazy_static;
use linkify::{Link, LinkFinder, LinkKind};
use regex::Regex;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::hook, StandardFramework},
    model::{
        gateway::Ready,
        prelude::{ChannelId, Message},
    },
    prelude::TypeMapKey,
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

struct PostableChannels;
impl TypeMapKey for PostableChannels {
    type Value = HashSet<ChannelId>;
}

/// Scrape the ChannelId and Link from a string input
fn scrape_message<'a>(text: &'a str) -> Option<(ChannelId, &'a str)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<#(\d+)>").unwrap();
        static ref FINDER: LinkFinder = {
            let mut finder = LinkFinder::new();
            finder.kinds(&[LinkKind::Url]);

            finder
        };
    }

    let links: Vec<Link> = FINDER.links(text).collect();
    if links.len() <= 0 {
        return None;
    }

    let captures = match RE.captures(text) {
        Some(captures) => captures,
        None => return None,
    };
    let capture = match captures.get(1) {
        Some(capture) => capture,
        None => return None,
    };
    if let Ok(id) = capture.as_str().parse::<u64>() {
        Some((ChannelId(id), links[0].as_str()))
    } else {
        None
    }
}

#[hook]
async fn normal_message_hook(ctx: &Context, msg: &Message) {
    let data = ctx.data.read().await;
    let postable_channels = data
        .get::<PostableChannels>()
        .expect("Expected PostableChannels in TypeMap");
    if let Some((channel_id, link)) = scrape_message(&msg.content) {
        if postable_channels.get(&channel_id).is_some() {
            channel_id
                .send_message(&ctx.http, |m| {
                    m.content(link);

                    m
                })
                .await
                .unwrap();
        }
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

    let framework = StandardFramework::new().normal_message(normal_message_hook);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client.");
    {
        let mut data = client.data.write().await;
        data.insert::<PostableChannels>(postable_channels);
    }

    if let Err(err) = client.start().await {
        eprintln!("Client error: {:?}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_grabs_channel_link() {
        let result =
            scrape_message("Check out this new game! https://kickstarter.com/ <#123456789>");

        assert!(result.is_some());
        if let Some((channel_id, link)) = result {
            assert_eq!(123456789, *channel_id.as_u64());
            assert_eq!("https://kickstarter.com/", link);
        }
    }

    #[test]
    fn it_returns_none_if_no_link() {
        let result = scrape_message("Check out this new game! <#123456789>");
        assert!(result.is_none());
    }

    #[test]
    fn it_returns_none_if_no_channel() {
        let result = scrape_message("Check out this new game! https://kickstarter.com/");
        assert!(result.is_none());
    }
}
