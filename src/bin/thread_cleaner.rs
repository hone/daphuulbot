use chrono::{offset::Utc, Duration};
use serenity::{
    http::client::Http,
    model::{channel::GuildChannel, id::ChannelId},
};
use std::{env, sync::Arc, thread};

const SLEEP_TIME_MS: u64 = 60 * 5;

/// Determine if a channel should be archived
fn should_archive(
    channel: &GuildChannel,
    thread_category_id: ChannelId,
    threshold: Duration,
) -> bool {
    if let Some(category_id) = channel.category_id {
        let last_timestamp = channel
            .last_message_id
            .map(|msg| msg.created_at())
            .unwrap_or_else(|| channel.id.created_at());
        return category_id == thread_category_id
            && Utc::now().signed_duration_since(last_timestamp) > threshold;
    }

    false
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Please set DISCORD_TOKEN as an env var.");
    let guild_id = env::var("DISCORD_GUILD")
        .map(|id| id.parse::<u64>().expect("Guild ID needs to be a u64."))
        .expect("Please set DISCORD_GUILD as an env var.");
    let thread_category_id = env::var("DISCORD_THREAD_CATEGORY")
        .map(|id| ChannelId(id.parse::<u64>().expect("Category ID needs to be a u64.")))
        .expect("Please set DISCORD_THREAD_CATEGORY");
    let archive_category_id = env::var("DISCORD_ARCHIVE_CATEGORY")
        .map(|id| ChannelId(id.parse::<u64>().expect("Category ID needs to be a u64.")))
        .expect("Please set DISCORD_ARCHIVE_CATEGORY");
    let threshold = env::var("THREAD_DURATION_IN_WEEKS")
        .map(|num| Duration::weeks(num.parse::<i64>().expect("Weeks need to be a i64.")))
        .expect("Please set THREAD_DURATION_IN_WEEKS");
    let http = Arc::new(Http::new_with_token(&token));

    loop {
        if let Ok(mut channels) = http.get_channels(guild_id).await {
            let archive_channels = channels
                .iter_mut()
                .filter(|channel| should_archive(channel, thread_category_id, threshold));
            for channel in archive_channels {
                println!("Archiving Channel: {}", channel.name);

                if let Err(err) = channel
                    .edit(&http, |c| c.category(archive_category_id))
                    .await
                {
                    eprintln!("Http error: {:?}", err);
                }
            }
        }

        thread::sleep(std::time::Duration::from_secs(SLEEP_TIME_MS));
    }
}
