#![deny(unused_must_use)]

use serenity::{
    async_trait, client::Context, client::EventHandler, model::channel::Channel,
    model::channel::ChannelType, model::id::ChannelId, model::prelude::Ready, Client,
};
use std::env;

const CHANNEL_ID: ChannelId = ChannelId(717435160378867772);

struct Handler;

// #[async_trait]
impl EventHandler for Handler {
    // async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
    //     let channel = CHANNEL_ID
    //         .to_channel(ctx)
    //         .await
    //         .expect("The channel stored in CHANNEL_ID does not exist");

    //     if let Channel::Guild(channel) = channel {
    //         if channel.kind == ChannelType::Voice {
    //             println!("{:#?}", channel.bitrate);
    //         } else {
    //             panic!("The channel provided was not a guild voice channel, check the channel id");
    //         }
    //     } else {
    //         panic!("The channel provided was not a guild channel, check the channel id");
    //     }
    // }
}

#[async_std::main]
async fn main() {
    let token =
        env::var("DISCORD_TOKEN").expect("You must provide a DISCORD_TOKEN environment variable");

    let mut client = Client::new(token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    // ctx.lock().await.shutdown_all();
}
