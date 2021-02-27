use lazy_static::lazy_static;
use linkify::{Link, LinkFinder, LinkKind};
use regex::Regex;
use serenity::{
    client::Context,
    framework::standard::macros::hook,
    model::prelude::{ChannelId, Message},
    prelude::TypeMapKey,
};
use std::collections::HashSet;

mod kickstarter;

pub struct PostableChannels;
impl TypeMapKey for PostableChannels {
    type Value = HashSet<ChannelId>;
}

#[hook]
pub async fn normal_message_hook(ctx: &Context, msg: &Message) {
    let data = ctx.data.read().await;
    let postable_channels = data
        .get::<PostableChannels>()
        .expect("Expected PostableChannels in TypeMap");
    if let Some((channel_id, link)) = scrape_message(&msg.content) {
        if postable_channels.get(&channel_id).is_some() {
            if let Ok(info) = kickstarter::EmbedInfo::from_url(link).await {
                channel_id
                    .send_message(&ctx.http, |m| {
                        m.content(link);

                        m.embed(|e| {
                            e.title(info.title);
                            e.description(info.description);
                            e.url(info.url);
                            e.image(info.image);

                            e
                        });

                        m
                    })
                    .await
                    .unwrap();
            } else {
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
