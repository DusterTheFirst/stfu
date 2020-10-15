//! Constants shared across the program

use rarity_permission_calculator::prelude::Permissions; // TODO: change once v2 hits

/// The required permissions for the bot to function
pub const REQUIRED_PERMISSIONS: Permissions = Permissions::from_bits_truncate(
    Permissions::MUTE_MEMBERS.bits()
        | Permissions::DEAFEN_MEMBERS.bits()
        | Permissions::VIEW_CHANNEL.bits(),
);

/// Defaults exposed to the rest of the server, can probably be override with an environment variable
pub mod defaults {
    use twilight_model::id::ApplicationId;

    /// The client id of the oauth application
    pub const CLIENT_ID: ApplicationId = ApplicationId(746_070_136_980_766_861);

    /// The oauth redirect urls to choose from
    pub const REDIRECT_URLS: &[&str] = &["http://localhost:8000/oauth/done"];
}
