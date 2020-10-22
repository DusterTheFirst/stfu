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
use consts::{oauth, AUTH_COOKIE_NAME, BACKEND_URL, FRONTEND_URL};
use content::Html;
use graphql::{create_schema, DiscordContext, GraphQLContext, Schema};
use juniper_rocket::{graphiql_source, GraphQLRequest, GraphQLResponse};
use log::{error, info, trace, warn};
use reqwest::{header::HeaderMap, Client as ReqwestClient};
use rocket::{
    config::Environment,
    http::Status,
    http::{Cookie, Cookies, RawStr},
    request::FromRequest,
    request::Outcome,
    response::Debug,
    response::{content, Redirect},
    routes, uri, Config, State,
};
use rocket_cors::{Cors, CorsOptions};
use serde::{Deserialize, Serialize};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::shard::ShardBuilder;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_modelv1::id::ApplicationId;
use twilight_oauth2::{
    request::access_token_exchange::AccessTokenExchangeResponse, Client as OauthClient, Prompt,
    Scope,
};

pub mod consts;
pub mod graphql;

#[rocket::get("/", rank = 0)]
fn graphiql(_user: OauthUser) -> Html<String> {
    graphiql_source("/graphql")
}

#[rocket::get("/", rank = 1)]
fn graphiql_no_auth() -> Redirect {
    Redirect::to(uri!(oauth_login: "/"))
}

#[rocket::get("/graphql?<request>")]
#[allow(clippy::needless_pass_by_value)]
fn get_graphql_handler(
    discord: State<DiscordContext>,
    oauth: OauthUser,
    schema: State<Schema>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request.execute(
        &schema,
        &GraphQLContext {
            discord: discord.clone(),
            oauth,
        },
    )
}

#[rocket::post("/graphql", data = "<request>")]
#[allow(clippy::needless_pass_by_value)]
fn post_graphql_handler(
    discord: State<DiscordContext>,
    oauth: OauthUser,
    schema: State<Schema>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request.execute(
        &schema,
        &GraphQLContext {
            discord: discord.clone(),
            oauth,
        },
    )
}

#[rocket::get("/oauth/login?<from>")]
#[allow(clippy::needless_pass_by_value)]
fn oauth_login(discord: State<DiscordContext>, from: Option<String>) -> Redirect {
    let authorization_url = discord
        .oauth
        .authorization_url(oauth::REDIRECT_URL)
        .unwrap()
        .scopes(&[Scope::Identify])
        .prompt(Prompt::None)
        .state(&urlencoding::encode(
            &from.unwrap_or_else(|| "/".to_string()),
        ))
        .build();

    Redirect::to(authorization_url)
}

#[rocket::get("/oauth/authorize?<code>&<state>")]
#[allow(clippy::needless_pass_by_value)]
fn oauth_authorize(
    discord: State<DiscordContext>,
    reqwest: State<ReqwestClient>,
    code: String,
    state: &RawStr,
    mut cookies: Cookies,
) -> Result<Redirect, Debug<anyhow::Error>> {
    // FIXME: Better error page
    let mut request = discord.oauth.access_token_exchange(code.as_ref());
    let request = request.scopes(&[Scope::Identify]).build();

    info!("{}", serde_json::to_string_pretty(&request.body).unwrap());

    let response: Result<AccessTokenExchangeResponse, anyhow::Error> = task::block_on(async {
        Ok(reqwest
            .post(&request.url())
            .headers(request.headers.into_iter().fold(
                HeaderMap::new(),
                |mut map, (header, value)| {
                    map.append(*header, value.parse().unwrap());
                    map
                },
            ))
            .form(&request.body)
            .send()
            .await
            .context("Failed to make request")?
            .text()
            .await
            .context("Failed to read in response from request")?)
    })
    .and_then(|res| {
        Ok(serde_json::from_str(&res).context("Failed to parse the response from the request")?)
    });

    match response {
        Ok(response) => {
            cookies.add_private(Cookie::new(
                AUTH_COOKIE_NAME,
                serde_json::to_string(&OauthCookie::from(response))
                    .expect("Oauth cookie was unable to be serialized"),
            ));

            Ok(Redirect::to(state.url_decode_lossy()))
        }
        Err(e) => {
            error!("{:?}", e);

            Err(Debug(e))
            // Redirect::to(format!("{}?error={}", FRONTEND_URL, e.to_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OauthCookie {
    /// Access token to be used when making requests to the API on the user's
    /// behalf.
    pub access_token: String,
    /// Number of seconds from issuing that the access token is valid.
    ///
    /// After this duration, the refresh token must be exchanged for another
    /// access token and refresh token pair.
    pub expires_in: u64,
    /// Refresh token to use to exchange for another access token and refresh
    /// token pair.
    pub refresh_token: String,
    /// The seconds since the unix epoch that this oauth token was created
    pub created_at: u64,
}

impl From<AccessTokenExchangeResponse> for OauthCookie {
    fn from(
        AccessTokenExchangeResponse {
            access_token,
            expires_in,
            refresh_token,
            ..
        }: AccessTokenExchangeResponse,
    ) -> Self {
        OauthCookie {
            access_token,
            expires_in,
            refresh_token,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        }
    }
}

#[derive(Debug)]
pub struct OauthUser {
    pub http: HttpClient,
    pub auth: OauthCookie,
}

impl<'a, 'r> FromRequest<'a, 'r> for OauthUser {
    type Error = serde_json::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> Outcome<Self, Self::Error> {
        let cookie = request.cookies().get_private(AUTH_COOKIE_NAME);

        dbg!(&cookie);

        if let Some(cookie) = cookie {
            match serde_json::from_str::<OauthCookie>(cookie.value()) {
                Ok(cookie) => {
                    // TODO: Auto refresh if time is neigh
                    Outcome::Success(OauthUser {
                        http: HttpClient::new(cookie.access_token.clone()),
                        auth: cookie,
                    })
                }
                Err(e) => Outcome::Failure((Status::BadRequest, e)),
            }
        } else {
            Outcome::Forward(())
        }
    }
}

// #[rocket::get("/oauth/logout", data = "<request>")]
// #[allow(clippy::needless_pass_by_value)]
// fn post_graphql_handler(
//     context: State<DiscordContext>,
//     schema: State<Schema>,
//     request: GraphQLRequest,
// ) -> GraphQLResponse {
//     request.execute(&schema, &context)
// }

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
            oauth::CLIENT_ID
        });

    let oauth = OauthClient::new(client_id, client_secret, &[oauth::REDIRECT_URL])?;

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
            graphiql,
            graphiql_no_auth,
            get_graphql_handler,
            post_graphql_handler,
            oauth_login,
            oauth_authorize
        ],
    )
    .attach(
        Cors::from_options(&CorsOptions {
            // allow_credentials: true,
            ..CorsOptions::default()
        })
        .context("Failed to setup cors")?,
    )
    .launch(); // FIXME: Error handling, and json only responses

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
