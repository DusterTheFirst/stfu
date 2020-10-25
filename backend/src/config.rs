//! The configuration for the app

use serde::Deserialize;

/// The configuration for the application
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Token to authenticate the bot with discord
    #[serde(rename = "discord_token")]
    pub token: String,
    /// Oauth client secret
    pub client_secret: String,
    /// Oauth redirect url
    pub redirect_url: String,
    /// Name of the authentication cookie
    pub auth_cookie_name: String,
    /// Proxy url to use
    #[cfg(feature = "mitm_proxy")]
    pub proxy_url: String,
    /// Proxy certificate path to use
    #[cfg(feature = "mitm_proxy")]
    pub proxy_cert_path: String,
}
