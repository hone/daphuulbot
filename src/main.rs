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
            macros::{group, help},
            Args, CommandGroup, CommandResult, HelpOptions,
        },
        StandardFramework,
    },
    model::{
        gateway::Ready,
        id::EmojiId,
        prelude::{ChannelId, Message, UserId},
    },
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

#[group]
#[commands(thread)]
struct General;

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
    let thread_category = env::var("DISCORD_THREAD_CATEGORY")
        .map(|id| ChannelId(id.parse::<u64>().expect("Category IDs must be u64s")))
        .expect("Please set DISCORD_THREAD_CATEGORY");
    let emoji_yes_id = env::var("EMOJI_YES")
        .map(|id| EmojiId(id.parse::<u64>().expect("Emoji IDs must be u64s")))
        .expect("Please set EMOJI_YES");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .normal_message(feed::normal_message_hook)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let http = serenity::http::client::Http::new_with_token(&token);
    let guild = serenity::model::id::GuildId(739994825964912714);
    let emoji_yes = guild
        .emojis(&http)
        .await
        .unwrap()
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
        eprintln!("Client error: {:?}", err);
    }
}
