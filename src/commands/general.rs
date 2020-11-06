use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::ChannelType,
        prelude::{ChannelId, Message},
    },
    prelude::TypeMapKey,
    utils::MessageBuilder,
};

pub struct ThreadCategory;
impl TypeMapKey for ThreadCategory {
    type Value = ChannelId;
}

#[command]
#[num_args(1)]
#[usage = "<thread_name>"]
#[example = "hypehypehype"]
/// Create a new thread
pub async fn thread(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>()?;
    let data = ctx.data.read().await;
    let thread_category = data
        .get::<ThreadCategory>()
        .expect("Expected ThreadCategory in TypeMap");

    let guild = match msg.guild(&ctx.cache).await {
        Some(g) => g,
        None => return Ok(()),
    };
    let channel = guild
        .create_channel(&ctx.http, |c| {
            c.name(name)
                .kind(ChannelType::Text)
                .category(thread_category)
        })
        .await?;

    msg.reply(
        &ctx.http,
        MessageBuilder::new()
            .push("Created new thread ")
            .channel(channel)
            .push(".")
            .build(),
    )
    .await?;

    Ok(())
}
