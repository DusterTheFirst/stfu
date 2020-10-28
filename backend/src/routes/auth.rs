//! The authentication based routes and handlers

#![allow(clippy::needless_pass_by_value, clippy::must_use_candidate)]

use crate::{
    auth::OauthCookie, auth::OauthUser, config::Config, consts::OAUTH_SCOPES,
    graphql::DiscordContext, templates::HtmlRedirect,
};
use anyhow::Context;
use reqwest::{header::HeaderMap, Client as ReqwestClient};
use rocket::{
    http::SameSite,
    http::{Cookie, CookieJar, RawStr},
    response::Debug,
    response::{content::Html, Redirect},
    State,
};
use twilight_oauth2::{request::access_token_exchange::AccessTokenExchangeResponse, Prompt};

/// The login route for the oauth authentication
#[rocket::get("/oauth/login?<from>")]
pub fn oauth_login(
    discord: State<DiscordContext>,
    config: State<Config>,
    from: String,
) -> Redirect {
    let authorization_url = discord
        .oauth
        .authorization_url(&config.redirect_url)
        .expect("Redirect url is not one of the allowed")
        .scopes(OAUTH_SCOPES)
        .prompt(Prompt::None)
        .state(&urlencoding::encode(&from))
        .build();

    Redirect::to(authorization_url)
}

/// The authorize callback route for the oauth flow
#[rocket::get("/oauth/authorize?<code>&<state>")]
pub async fn oauth_authorize<'r>(
    discord: State<DiscordContext, 'r>,
    reqwest_client: State<ReqwestClient, 'r>,
    config: State<Config, 'r>,
    code: String,
    state: &RawStr,
    cookies: &CookieJar<'r>,
) -> Result<HtmlRedirect, Debug<anyhow::Error>> {
    // FIXME: Better error page
    let mut request = discord
        .oauth
        .access_token_exchange(code.as_ref(), &config.redirect_url)
        .unwrap();
    let request = request.scopes(OAUTH_SCOPES).build();

    let response =
        reqwest_client
            .post(&request.url())
            .headers(request.headers.iter().fold(
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
            .error_for_status()
            .context("Received an error from the server")?
            .text()
            .await
            .context("Failed to read in response from request")?;

    let response: AccessTokenExchangeResponse =
        serde_json::from_str(&response).context("Failed to parse the response from the request")?;

    cookies.add_private(
        Cookie::build(
            config.auth_cookie_name.clone(),
            serde_json::to_string(&OauthCookie::from(response))
                .context("Oauth cookie was unable to be serialized")?,
        )
        .domain(config.auth_cookie_domain.clone())
        .same_site(SameSite::Lax)
        .finish(),
    );

    Ok(HtmlRedirect {
        url: state.url_decode_lossy(),
    })
}

/// The oauth callback route in case of error
#[rocket::get("/oauth/authorize?<error>&<error_description>&<state>", rank = 1)]
pub async fn oauth_authorize_failure<'r>(
    error: String,
    error_description: String,
    state: &RawStr,
) -> Html<String> {
    // FIXME: better error. Flash cookies?
    Html(format!("<h1>{error}</h1><pre>{error_description}</pre><a href=\"{back}\">Go back to <b>{back}<b></a>", error=error, error_description=error_description, back=state.url_decode_lossy()))
}

/// The oauth route for logging out
#[rocket::get("/oauth/logout?<from>")]
pub async fn oauth_logout<'r>(
    // reqwest_client: State<ReqwestClient, 'r>,
    _oauth: OauthUser, //TODO: Revoke url
    from: &RawStr,
    config: State<Config, 'r>,
    cookies: &CookieJar<'r>,
) -> Result<HtmlRedirect, Debug<anyhow::Error>> {
    // FIXME: Better error page

    cookies.remove_private(Cookie::named(config.auth_cookie_name.clone()));

    Ok(HtmlRedirect {
        url: dbg!(from.url_decode_lossy()),
    })
}

/// The oauth route for logging out if already logged out
#[rocket::get("/oauth/logout?<from>", rank = 1)]
pub fn oauth_logout_not_logged_in(from: &RawStr) -> HtmlRedirect {
    HtmlRedirect {
        url: dbg!(from.url_decode_lossy()),
    }
}
