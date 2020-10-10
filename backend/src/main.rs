//! An elaborate, over-engineered solution to shutting my friends up

#![deny(unused_must_use)]
#![warn(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![feature(decl_macro, proc_macro_hygiene)]

use anyhow::Context;
use async_std::{stream::StreamExt, task};
use content::Html;
use graphql::{create_schema, DiscordContext, Schema};
use juniper_rocket::{graphiql_source, GraphQLRequest, GraphQLResponse};
use log::{debug, error, info, trace, warn};
use rocket::{config::Environment, response::content, routes, Config, State};
use std::env;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{shard::ShardBuilder, Event};
use twilight_http::Client as HttpClient;
use twilight_model::{gateway::Intents, id::ChannelId};

pub mod consts;
pub mod graphql;

const CHANNEL_ID: ChannelId = ChannelId(717435160378867772);

// struct Handler;

// #[async_trait]
// impl EventHandler for Handler {
//     async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
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
//     }
// }

#[rocket::get("/")]
fn graphiql() -> Html<String> {
    graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    context: State<DiscordContext>,
    request: GraphQLRequest,
    schema: State<Schema>,
) -> GraphQLResponse {
    request.execute(&schema, context.inner())
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    context: State<DiscordContext>,
    request: GraphQLRequest,
    schema: State<Schema>,
) -> GraphQLResponse {
    request.execute(&schema, context.inner())
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env::set_var(
        "RUST_LOG",
        "warn,_=info,launch=info,launch_=info,rocket=info",
    );
    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN")
        .context("You must provide a DISCORD_TOKEN environment variable")?;

    let http = HttpClient::new(&token);

    let mut shard = ShardBuilder::new(token)
        .http_client(http.clone())
        .intents(
            Intents::GUILDS
                | Intents::GUILD_VOICE_STATES
                | Intents::GUILD_MEMBERS
                | Intents::GUILD_PRESENCES,
        ) // FIXME: use only ones needed
        .build();
    shard.start().await?;

    let cache = InMemoryCache::new();

    // Startup an event loop for each event in the event stream
    {
        let shard = shard.clone();
        let cache = cache.clone();
        task::spawn(async move {
            let mut events = shard.events();

            while let Some(event) = events.next().await {
                cache.update(&event);
                if let Event::Ready(ready) = &event {
                    dbg!(&ready.guilds);
                }
                if let Event::GuildUpdate(update) = &event {
                    dbg!(&update.name);
                }
                if let Event::GuildCreate(create) = &event {
                    dbg!(
                        &create.members,
                        cache.guild(create.id),
                        cache.guild_members(create.id)
                    );
                }
            }
        });
    }

    #[cfg(debug_attributes)]
    {
        let mut shard = shard.clone();
        ctrlc::set_handler(move || {
            shard.shutdown();
            std::process::exit(1);
        });
    }

    rocket::custom(
        Config::build(Environment::active()?)
            .address("127.0.0.1")
            .unwrap(),
    )
    // .manage(Database::new())
    .manage(DiscordContext { cache, http, shard })
    .manage(create_schema())
    .mount(
        "/",
        routes![graphiql, get_graphql_handler, post_graphql_handler],
    )
    .launch();

    Ok(())
}
