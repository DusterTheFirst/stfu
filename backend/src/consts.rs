//! Constants shared across the program

use twilight_oauth2::Scope;
use twilight_permission_calculator::prelude::Permissions; // TODO: change once v2 hits

/// The required permissions for the bot to function
pub const REQUIRED_PERMISSIONS: Permissions = Permissions::from_bits_truncate(
    Permissions::MUTE_MEMBERS.bits()
        | Permissions::DEAFEN_MEMBERS.bits()
        | Permissions::VIEW_CHANNEL.bits(),
);

/// The oauth scopes to ask for
pub const OAUTH_SCOPES: &[Scope] = &[Scope::Identify, Scope::Guilds];

/// The oauth redirect urls that are allowed
pub const OAUTH_REDIRECT_URLS: &[&str] = &[
    "http://localhost:8000/oauth/authorize",
    "https://stfu-backend.dusterthefirst.com/oauth/authorize",
];
