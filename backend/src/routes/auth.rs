//! The authentication based routes and handlers

#![allow(clippy::needless_pass_by_value)]

use crate::{
    auth::OauthCookie,
    consts::{
        oauth::{REDIRECT_URL, SCOPES},
        AUTH_COOKIE_NAME,
    },
    graphql::DiscordContext,
};
use anyhow::Context;
use log::trace;
use reqwest::{header::HeaderMap, Client as ReqwestClient};
use rocket::{
    http::{Cookie, CookieJar, RawStr},
    response::Debug,
    response::{content::Html, Redirect},
    State,
};
use twilight_oauth2::{request::access_token_exchange::AccessTokenExchangeResponse, Prompt};

/// The login route for the oauth authentication
#[rocket::get("/oauth/login?<from>")]
pub fn oauth_login(discord: State<DiscordContext>, from: Option<String>) -> Redirect {
    let authorization_url = discord
        .oauth
        .authorization_url(REDIRECT_URL)
        .unwrap()
        .scopes(SCOPES)
        .prompt(Prompt::Consent)
        .state(&urlencoding::encode(
            &from.unwrap_or_else(|| "/".to_string()),
        ))
        .build();

    Redirect::to(authorization_url)
}

/// The authorize callback route for the oauth flow
#[rocket::get("/oauth/authorize?<code>&<state>")]
pub async fn oauth_authorize<'r>(
    discord: State<DiscordContext, 'r>,
    reqwest: State<ReqwestClient, 'r>,
    code: String,
    state: &RawStr,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Debug<anyhow::Error>> {
    // FIXME: Better error page
    let mut request = discord.oauth.access_token_exchange(code.as_ref());
    let request = request.scopes(SCOPES).build();

    trace!("{}", serde_json::to_string_pretty(&request.body).unwrap());

    let response =
        reqwest
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
            .context("Failed to read in response from request")?;

    trace!("past response from discord: {}", response);

    let response: AccessTokenExchangeResponse =
        serde_json::from_str(&response).context("Failed to parse the response from the request")?;

    trace!("past response parsing");

    cookies.add_private(Cookie::new(
        AUTH_COOKIE_NAME,
        serde_json::to_string(&OauthCookie::from(response))
            .context("Oauth cookie was unable to be serialized")?,
    ));

    trace!(
        "past cookie add: redirecting to {}",
        state.url_decode_lossy()
    );

    Ok(Redirect::to(state.url_decode_lossy()))
}

/// The oauth callback route in case of error
#[rocket::get("/oauth/authorize?<error>&<error_description>&<state>", rank = 1)]
pub async fn oauth_authorize_failure<'r>(
    error: String,
    error_description: String,
    state: &RawStr,
) -> Html<String> {
    Html(format!("<h1>{error}</h1><pre>{error_description}</pre><a href=\"{back}\">Go back to <b>{back}<b></a>", error=error, error_description=error_description, back=state.percent_decode_lossy()))
}

// TODO:
// #[rocket::get("/oauth/logout", data = "<request>")]
// fn post_graphql_handler(
//     context: State<DiscordContext>,
//     schema: State<Schema>,
//     request: GraphQLRequest,
// ) -> GraphQLResponse {
//     request.execute(&schema, &context)
// }
