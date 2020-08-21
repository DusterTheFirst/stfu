use crate::args::{Args, ArgsKey, Command};
use notify_rust::Notification;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Channel, id::ChannelId, prelude::Ready},
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let data = ctx.data.read().await;

        let Args { command, channel, .. } = data.get::<ArgsKey>().unwrap();

        let channel = ChannelId::from(*channel)
            .to_channel(&ctx)
            .await
            .expect("The channel does not exist");

        if let Channel::Guild(channel) = channel {
            let members = channel
                .members(&ctx)
                .await
                .expect("Failed to get members in the channel");

            let mute = match command {
                Command::Mute(_) => true,
                Command::Unmute(_) => false,
            };

            for member in members {
                if !member.user.bot {
                    match member.edit(&ctx, |m| m.mute(mute)).await {
                        Err(e) => eprintln!("{}", e),
                        Ok(()) => {}
                    };
                }
            }

            Notification::new()
                .summary("Success")
                .body(match mute {
                    true => "Muted everyone",
                    false => "Unmuted everyone",
                })
                .show()
                .unwrap();
        } else {
            panic!("The channel provided was not a guild channel, check the channel id");
        }

        ctx.shard.shutdown_clean();
        std::process::exit(0);
    }
}
