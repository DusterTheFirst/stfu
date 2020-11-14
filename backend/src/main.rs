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
use config::Config;
use consts::OAUTH_REDIRECT_URLS;
use dotenv::dotenv;
use graphql::{create_schema, DiscordContext};
use log::warn;
use reqwest::Client as ReqwestClient;
use rocket::{
    figment::{providers::Env, Figment},
    http::Method,
    routes,
};
use rocket_cors::CorsOptions;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::shard::ShardBuilder;
use twilight_http::{client::ClientBuilder as HttpClientBuilder, Client as HttpClient};
use twilight_model::gateway::Intents;
use twilight_oauth2::Client as OauthClient;

pub mod auth;
pub mod config;
pub mod consts;
pub mod graphql;
pub mod routes;
pub mod templates;

#[cfg(all(feature = "mitm_proxy", not(debug_assertions)))]
compile_error!("You cannot have the `mitm_proxy` feature enabled in release mode");

/// Helper function to create a pre-configured discord http client
pub fn create_http_client(token: impl Into<String>, _config: &Config) -> HttpClient {
    let builder = HttpClientBuilder::new().token(token);

    #[cfg(feature = "mitm_proxy")]
    {
        warn!("Creating a http client with a connection to mitm proxy");

        builder
            .proxy(reqwest::Proxy::all(&_config.proxy_url).expect("Proxy url was malformed"))
            .reqwest_client(
                reqwest::ClientBuilder::new().add_root_certificate(
                    reqwest::Certificate::from_pem(
                        std::fs::read(&_config.proxy_cert_path)
                            .expect("Failed to read in proxy cert")
                            .as_slice(),
                    )
                    .expect("Certificate was in an invalid format"),
                ),
            )
            .build()
            .unwrap()
    }

    #[cfg(not(feature = "mitm_proxy"))]
    builder.build().unwrap()
}

/// Helper function to create a pre-configured reqwest client
#[must_use = "The created client must be used"]
pub fn create_reqwest_client(_config: &Config) -> ReqwestClient {
    #[cfg(feature = "mitm_proxy")]
    {
        warn!("Creating a http client with a connection to mitm proxy");

        ReqwestClient::builder()
            .proxy(reqwest::Proxy::all(&_config.proxy_url).expect("Proxy url was malformed"))
            .add_root_certificate(
                reqwest::Certificate::from_pem(
                    std::fs::read(&_config.proxy_cert_path)
                        .expect("Failed to read in proxy cert")
                        .as_slice(),
                )
                .expect("Certificate was in an invalid format"),
            )
            .build()
            .unwrap()
    }

    #[cfg(not(feature = "mitm_proxy"))]
    ReqwestClient::new()
}

#[cfg(not(feature = "generate_schema"))]
#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config: Config = envy::from_env().context("Missing required environment variables")?;

    pretty_env_logger::init();

    let http = create_http_client(&config.token, &config);

    let oauth = Arc::new(OauthClient::new(
        http.current_user_application().await?.id,
        &config.client_secret,
        OAUTH_REDIRECT_URLS,
    )?);

    let mut shard = ShardBuilder::new(
        &config.token,
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

    rocket::custom(Figment::from(rocket::Config::default()).merge(Env::prefixed("ROCKET_")))
        .manage(create_reqwest_client(&config))
        .manage(DiscordContext {
            cache,
            http,
            shard: shard.clone(),
            oauth,
        })
        .manage(config.clone())
        .manage(create_schema())
        .mount(
            "/",
            routes![
                routes::graphql::graphiql,
                routes::graphql::graphiql_no_auth,
                routes::graphql::get_graphql_handler,
                routes::graphql::get_graphql_no_auth,
                routes::graphql::post_graphql_handler,
                routes::graphql::post_graphql_no_auth,
                routes::auth::oauth_login,
                routes::auth::oauth_authorize,
                routes::auth::oauth_authorize_failure,
                routes::auth::oauth_logout,
                routes::auth::oauth_logout_not_logged_in
            ],
        )
        .attach(
            CorsOptions {
                allow_credentials: true,
                allowed_methods: [Method::Get, Method::Post]
                    .iter()
                    .cloned()
                    .map(rocket_cors::Method::from)
                    .collect(),
                // allowed_origins: AllOrSome::Some(()),
                ..CorsOptions::default()
            }
            .to_cors()
            .context("Failed to setup cors")?,
        )
        .launch()
        .await?; // FIXME: Error handling

    // After server has shutdown
    shard.shutdown();

    Ok(())
}

#[cfg(feature = "generate_schema")]
#[async_std::main] // FIXME: maybe move to test or build script
async fn main() {
    use twilight_http::Client as HttpClient;

    let http = HttpClient::new("");

    let shard = ShardBuilder::new("", Intents::empty())
        .http_client(http.clone())
        .build();

    let cache = InMemoryCache::new();

    let oauth = Arc::new(OauthClient::new(ApplicationId(1), "", &[]).unwrap());

    let (res, _errors) = juniper::introspect(
        &create_schema(),
        &graphql::GraphQLContext {
            discord: DiscordContext {
                cache,
                http,
                shard,
                oauth,
            },
            user: auth::OauthUser {
                http: HttpClient::new(""),
                auth: auth::OauthCookie {
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
        "{}/target/graphql.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    std::io::Write::write(
        &mut file,
        serde_json::to_string_pretty(&res).unwrap().as_bytes(),
    )
    .unwrap();
}
