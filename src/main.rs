use daphuulbot::{
    commands::{self, general::*},
    feed,
};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            help_commands,
            macros::{group, help, hook},
            Args, CommandGroup, CommandResult, HelpOptions,
        },
        StandardFramework,
    },
    model::{
        gateway::Ready,
        id::{EmojiId, GuildId},
        prelude::{ChannelId, Message, UserId},
    },
};
use std::{collections::HashSet, env};
use tracing::{error, info, instrument};

/// EventHandler for signaling when the bot is connected and ready.
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
#[lacking_role("hide")]
#[max_levenshtein_distance(3)]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
#[instrument]
async fn before(_ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    info!("Command: {command_name}");

    true
}

#[group]
#[commands(thread)]
struct General;

#[tokio::main]
#[instrument]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Please set DISCORD_TOKEN as an env var.");
    let guild = env::var("DISCORD_GUILD")
        .map(|id| GuildId(id.parse::<u64>().expect("Guild IDs must be u64s")))
        .expect("Please set DISCORD_GUILD");
    let postable_channels = env::var("DISCORD_POSTABLE_CHANNELS")
        .expect("Please set allowed DISCORD_POSTABLE_CHANNELS")
        .split(",")
        .map(|str| Ok(ChannelId(str.parse::<u64>()?)))
        .collect::<anyhow::Result<HashSet<ChannelId>>>()
        .expect("Channel IDs must be u64s");
    let thread_category = env::var("DISCORD_THREAD_CATEGORY")
        .map(|id| ChannelId(id.parse::<u64>().expect("Category IDs must be u64s")))
        .expect("Please set DISCORD_THREAD_CATEGORY");
    let emoji_yes_id = env::var("EMOJI_YES")
        .map(|id| EmojiId(id.parse::<u64>().expect("Emoji IDs must be u64s")))
        .expect("Please set EMOJI_YES");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .before(before)
        .normal_message(feed::normal_message_hook)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let http = serenity::http::client::Http::new_with_token(&token);
    let emojis = guild.emojis(&http).await.unwrap();
    for emoji in emojis.iter() {
        info!("Emoji: {emoji}");
    }
    let emoji_yes = emojis
        .into_iter()
        .find(|emoji| emoji.id == emoji_yes_id)
        .expect("Could not find EMOJI_YES in server emojis");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client.");
    {
        let mut data = client.data.write().await;
        data.insert::<feed::PostableChannels>(postable_channels);
        data.insert::<feed::Emoji>(emoji_yes);
        data.insert::<commands::general::ThreadCategory>(thread_category);
    }

    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    }
}
