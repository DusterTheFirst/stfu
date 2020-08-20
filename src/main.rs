use anyhow::{anyhow, Context, Result};
use std::env;
use tokio::stream::StreamExt;
use twilight::{
    cache::{
        twilight_cache_inmemory::config::{EventType, InMemoryConfigBuilder},
        InMemoryCache,
    },
    gateway::{
        cluster::config::ShardScheme, shard::config::ShardConfigBuilder, Cluster, ClusterConfig,
        Event, Shard,
    },
    http::Client,
    model::{
        channel::{Channel, GuildChannel},
        gateway::GatewayIntents,
        id::ChannelId,
    },
};

const CHANNEL_ID: ChannelId = ChannelId(717435160378867772);

#[tokio::main]
async fn main() -> Result<()> {
    let token = env::var("DISCORD_TOKEN")
        .context("You must provide a DISCORD_TOKEN environment variable")?;

    let http = Client::new(&token);

    let mut shard = Shard::new(
        ShardConfigBuilder::new(token)
            .http_client(http)
            .intents(Some(
                GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
            )),
    );
    shard.start().await?;

    let cache = InMemoryCache::from(
        InMemoryConfigBuilder::new()
            .event_types(
                EventType::VOICE_STATE_UPDATE | EventType::GUILD_UPDATE | EventType::GUILD_CREATE,
            )
            .build(),
    );

    // Startup an event loop for each event in the event stream
    let event_shard = shard.clone();
    tokio::spawn(async move {
        let mut events = event_shard.events();

        while let Some(event) = events.next().await {
            // Update the cache
            cache.update(&event).await.expect("Cache failed, OhNoe");

            println!("{:?}", event);
        }
    })
    .await;

    let channel = cache
        .guild_channel(CHANNEL_ID).await?
        .context("The channel stored in CHANNEL_ID does not exist")?;

    // if let Channel::Guild(GuildChannel::Voice(channel)) = channel {
    //     println!("{:#?}", channel);
    // } else {
    //     anyhow!("The channel provided was not a guild voice channel, check the channel id");
    // }

    shard.shutdown();

    Ok(())
}
