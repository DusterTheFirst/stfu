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

use anyhow::Context;
use async_std::{stream::StreamExt, task};
use consts::oauth::{CLIENT_ID, REDIRECT_URL};
use graphql::{create_schema, DiscordContext};
use log::warn;
use reqwest::Client as ReqwestClient;
use rocket::routes;
// use rocket_cors::{Cors, CorsOptions};
use std::{env, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::shard::ShardBuilder;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_model_v1::id::ApplicationId;
use twilight_oauth2::Client as OauthClient;

pub mod auth;
pub mod consts;
pub mod graphql;
pub mod routes;

#[cfg(not(feature = "generate_schema"))]
#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env::set_var(
        "RUST_LOG",
        "warn,_=info,launch=info,launch_=info,rocket=info,stfu_backend=trace",
    );
    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN")
        .context("You must provide a DISCORD_TOKEN environment variable")?;

    let client_secret = env::var("CLIENT_SECRET")
        .context("You must provide a CLIENT_SECRET environment variable")?;

    // TODO: config?
    let client_id = env::var("CLIENT_ID")
        .map(|id| ApplicationId(id.parse().unwrap()))
        .unwrap_or_else(|_| {
            warn!("No CLIENT_ID environment variable provided, defaulting to client id in consts");
            CLIENT_ID
        });

    let oauth = Arc::new(OauthClient::new(client_id, client_secret, &[REDIRECT_URL])?);

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

    rocket::ignite()
        .manage(
            ReqwestClient::builder()
                .user_agent("Discord Bot: STFU")
                .build()?,
        )
        .manage(DiscordContext {
            cache,
            http,
            shard,
            oauth,
        })
        .manage(create_schema())
        .mount(
            "/",
            routes![
                routes::graphql::graphiql,
                routes::graphql::graphiql_no_auth,
                routes::graphql::get_graphql_handler,
                routes::graphql::post_graphql_handler,
                routes::auth::oauth_login,
                routes::auth::oauth_authorize,
                routes::auth::oauth_authorize_failure
            ],
        )
        // .attach(
        //     Cors::from_options(&CorsOptions {
        //         // allow_credentials: true,
        //         ..CorsOptions::default()
        //     })
        //     .context("Failed to setup cors")?,
        // )
        .launch()
        .await
        .ok(); // FIXME: Error handling, and json only responses

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

    let oauth = OauthClient::new(ApplicationId(1), "", &[]).unwrap();

    let (res, _errors) = juniper::introspect(
        &create_schema(),
        &GraphQLContext {
            discord: DiscordContext {
                cache,
                http,
                shard,
                oauth,
            },
            user: OauthUser {
                http: HttpClient::new(""),
                auth: OauthCookie {
                    access_token: "".into(),
                    expires_in: 0,
                    refresh_token: "".into(),
                    created_at: 0,
                },
            },
        },
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
