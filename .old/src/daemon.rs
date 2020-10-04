use daemonize_me::Daemon;
use messages::{Acknowledgement, Action};
use notify_rust::Notification;
use panic::PanicInfo;
use serenity::{
    async_trait,
    client::{bridge::gateway::ShardManager, Client, Context, EventHandler},
    model::{
        channel::Channel,
        id::ChannelId,
        prelude::{Activity, OnlineStatus, Ready},
    },
    prelude::{Mutex, TypeMapKey},
};
use std::{env, panic, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::{prelude::*, time::delay_for};

mod consts;
mod messages;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        ctx.set_presence(
            Some(Activity::listening("to your loud ass")),
            OnlineStatus::Online,
        )
        .await;

        let data = ctx.data.read().await;
        let shard_manager = data.get::<ShardManagerKey>().unwrap();

        let mut listener = TcpListener::bind(consts::ADDRESS)
            .await
            .expect("Failed to bind to socket");

        loop {
            let (mut socket, _) = tokio::select! {
                connection = listener.accept() => { connection.unwrap() }
                _ = delay_for(Duration::from_secs(10 * 60)) => {
                    println!("No calls for 10 minutes, closing");

                    shard_manager.lock().await.shutdown_all().await;

                    break;
                }
            };

            let mut buf = vec![0; Action::max_size()];
            socket
                .read_exact(&mut buf)
                .await
                .expect("Failed to read Action in");

            let action: Action = bincode::deserialize(&buf).expect("Failed to deserialize Action");

            {
                let (channel, mute): (ChannelId, _) = match action {
                    Action::Mute(id) => (id.into(), true),
                    Action::Unmute(id) => (id.into(), false),
                };

                let channel = channel
                    .to_channel(&ctx)
                    .await
                    .expect("The channel does not exist");

                if let Channel::Guild(channel) = channel {
                    let members = channel
                        .members(&ctx)
                        .await
                        .expect("Failed to get members in the channel");

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
            }

            let data = bincode::serialize(&Acknowledgement::Success(action))
                .expect("Failed to serialize the acknowledgement to send");

            socket
                .write(&data)
                .await
                .expect("Failed to send the acknowledgement the the client");
        }
    }
}

struct ShardManagerKey;
impl TypeMapKey for ShardManagerKey {
    type Value = Arc<Mutex<ShardManager>>;
}

fn main() {
    panic::set_hook(Box::new(|info: &PanicInfo| {
        Notification::new()
            .summary("PANIC")
            .body(&format!("{}", info))
            .show()
            .unwrap();
        eprint!("{}", info);
    }));

    let daemon = Daemon::new().start();

    match daemon {
        Ok(_) => println!("Daemonized with success"),
        Err(e) => panic!("Error, {}", e),
    }

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        async_main().await;
    });
}

async fn async_main() {
    let args = env::args().collect::<Vec<_>>();

    let token = args
        .get(1)
        .expect("You must pass one argument to the daemon, and it must be the token");

    let mut client = Client::new(token)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    {
        client
            .data
            .write()
            .await
            .insert::<ShardManagerKey>(client.shard_manager.clone());
    }

    println!("Starting client");
    client.start().await.expect("Client failed to run");
}
