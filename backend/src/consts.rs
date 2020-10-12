//! Constants shared across the program

use twilight_model::guild::Permissions;

/// The required permissions for the bot to function
pub const REQUIRED_PERMISSIONS: Permissions = Permissions::from_bits_truncate(
    Permissions::MUTE_MEMBERS.bits()
        | Permissions::DEAFEN_MEMBERS.bits()
        | Permissions::VIEW_CHANNEL.bits(),
);

pub mod defaults {
    use twilight_model::id::ApplicationId;

    pub const CLIENT_ID: ApplicationId = ApplicationId(746070136980766861);
    pub const REDIRECT_URLS: &[&str] = &["http://localhost:8000/oauth/done"];
}
