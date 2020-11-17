//! Structures and other tools used for authentication

use crate::{config::Config, create_http_client};
use anyhow::anyhow;
use log::warn;
use rocket::{
    http::{Cookie, Status},
    request::{FromRequest, Outcome},
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use twilight_http::Client as HttpClient;
use twilight_model::id::UserId;
use twilight_oauth2::request::access_token_exchange::AccessTokenExchangeResponse;

/// The cookie containing oauth authentication information for a user
///
/// The cookie can be derived from an `AccessTokenExchangeResponse`
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
    /// The user's id
    pub user_id: UserId,
}

impl OauthCookie {
    /// Create an instance of the cookie from the response and make a request to get the user id
    pub async fn create(
        AccessTokenExchangeResponse {
            access_token,
            expires_in,
            refresh_token,
            ..
        }: AccessTokenExchangeResponse,
        config: &Config,
    ) -> Result<Self, twilight_http::Error> {
        Ok(OauthCookie {
            user_id: create_http_client(format!("Bearer {}", access_token), config)
                .current_user()
                .await?
                .id,
            access_token,
            expires_in,
            refresh_token,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        })
    }
}

/// An authenticated oauth user
#[derive(Debug)]
pub struct OauthUser {
    /// The http client to request information from discord
    pub http: HttpClient,
    /// The authentication information stored in the cookie for the user
    pub cookie: OauthCookie,
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for OauthUser {
    type Error = anyhow::Error;

    async fn from_request(request: &'a rocket::Request<'r>) -> Outcome<Self, Self::Error> {
        let config: &Config = match request.managed_state() {
            Some(config) => config,
            None => {
                return Outcome::Failure((
                    Status::InternalServerError,
                    anyhow!("Config was not mounted on the rocket"),
                ))
            }
        };

        let cookie = request.cookies().get_private(&config.auth_cookie_name);

        if let Some(cookie) = cookie {
            match serde_json::from_str::<OauthCookie>(cookie.value()) {
                Ok(cookie) => {
                    // FIXME: Auto refresh if time is neigh

                    Outcome::Success(OauthUser {
                        http: create_http_client(format!("Bearer {}", cookie.access_token), config),
                        cookie,
                    })
                }
                Err(e) => {
                    warn!("Received malformed cookie. Clearing it. {}", e);

                    // Remove cookie if malformed
                    request
                        .cookies()
                        .remove_private(Cookie::named(config.auth_cookie_name.clone()));

                    Outcome::Forward(())
                }
            }
        } else {
            Outcome::Forward(())
        }
    }
}
