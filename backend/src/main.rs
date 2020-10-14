//! An elaborate, over-engineered solution to shutting my friends up

#![deny(unused_must_use)]
#![warn(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![cfg_attr(feature = "generate_schema", allow(unused_imports))]
#![feature(decl_macro, proc_macro_hygiene)]

use anyhow::Context;
use async_std::{stream::StreamExt, task};
use consts::defaults;
use content::Html;
use graphql::{create_schema, DiscordContext, Schema};
use juniper_rocket::{graphiql_source, GraphQLRequest, GraphQLResponse};
use log::{debug, error, info, trace, warn};
use rocket::{config::Environment, response::content, routes, Config, State};
use rocket_cors::{Cors, CorsOptions};
use std::env;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::shard::ShardBuilder;
use twilight_http::Client as HttpClient;
use twilight_model::{gateway::Intents, id::ApplicationId};

pub mod consts;
pub mod graphql;

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

#[cfg(not(feature = "generate_schema"))]
#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env::set_var(
        "RUST_LOG",
        "warn,_=info,launch=info,launch_=info,rocket=info",
    );
    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN")
        .context("You must provide a DISCORD_TOKEN environment variable")?;

    // let client_secret = env::var("CLIENT_SECRET")
    //     .context("You must provide a CLIENT_SECRET environment variable")?;

    // // TODO: config?
    // let client_id = env::var("CLIENT_ID").map(|id| ApplicationId(id.parse().unwrap())).unwrap_or_else(|_| {
    //     warn!("No CLIENT_ID environment variable provided, defaulting to client id in consts");
    //     defaults::CLIENT_ID
    // });
    // let redirect_uris = env::var("REDIRECT_URLS")
    //     .map(|all| all.split(",").map(|url| url.trim()).collect::<Vec<_>>())
    //     .unwrap_or_else(|_| {
    //         warn!("No REDIRECT_URLS environment variable provided, defaulting to redirect urls in consts");
    //         defaults::REDIRECT_URLS.to_vec()
    //     });

    // let oauth = twilight_oauth2::Client::new(client_id, client_secret, redirect_uris.as_slice());

    let http = HttpClient::new(&token);

    let mut shard = ShardBuilder::new(
        token,
        Intents::GUILDS
            | Intents::GUILD_VOICE_STATES
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_PRESENCES,
    )
    .http_client(http.clone())
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
    .attach(
        Cors::from_options(&CorsOptions {
            // allow_credentials: true,
            ..Default::default()
        })
        .context("Failed to setup cors")?,
    )
    .launch();

    Ok(())
}

#[cfg(feature = "generate_schema")]
#[async_std::main] // FIXME: maybe move to test or build script
async fn main() {
    let http = HttpClient::new("");

    let shard = ShardBuilder::new("", Intents::empty())
        .http_client(http.clone())
        .build();

    let cache = InMemoryCache::new();

    let (res, _errors) = juniper::introspect(
        &create_schema(),
        &DiscordContext { cache, http, shard },
        Default::default(),
    )
    .unwrap();

    let mut file = std::fs::File::create(format!(
        "{}/../target/graphql.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    std::io::Write::write(
        &mut file,
        serde_json::to_string_pretty(&res).unwrap().as_bytes(),
    )
    .unwrap();
}
