//! Constants shared across the program

use twilight_permission_calculator::prelude::Permissions; // TODO: change once v2 hits

/// The required permissions for the bot to function
pub const REQUIRED_PERMISSIONS: Permissions = Permissions::from_bits_truncate(
    Permissions::MUTE_MEMBERS.bits()
        | Permissions::DEAFEN_MEMBERS.bits()
        | Permissions::VIEW_CHANNEL.bits(),
);

/// The name of the cookie used to store auth data
pub const AUTH_COOKIE_NAME: &str = "stfu-auth";

/// The front end's url
#[cfg(not(debug_assertions))]
pub const FRONTEND_URL: &str = "https://stfu.duserthefirst.com";

/// The front end's url
#[cfg(debug_assertions)]
pub const FRONTEND_URL: &str = "http://localhost:3000";

/// The back end's url
#[cfg(not(debug_assertions))]
pub const BACKEND_URL: &str = "https://stfu-backend.duserthefirst.com";

/// The back end's url
#[cfg(debug_assertions)]
pub const BACKEND_URL: &str = "http://localhost:8000";

/// Defaults exposed to the rest of the server, can probably be override with an environment variable
pub mod oauth {
    use twilight_model_v1::id::ApplicationId;
    use twilight_oauth2::Scope;

    /// The client id of the oauth application
    pub const CLIENT_ID: ApplicationId = ApplicationId(746_070_136_980_766_861);

    /// The oauth scopes to ask for
    pub const SCOPES: &[Scope] = &[Scope::Identify, Scope::Guilds];

    /// The oauth redirect url
    #[cfg(debug_assertions)]
    pub const REDIRECT_URL: &str = "http://localhost:8000/oauth/authorize";

    /// The oauth redirect url
    #[cfg(not(debug_assertions))]
    pub const REDIRECT_URL: &str = "http://stfu-backend.dusterthefirst.com/oauth/authorize";
}
