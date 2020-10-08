use twilight_model::guild::Permissions;

/// The required permissions for the bot to function
pub const REQUIRED_PERMISSIONS: Permissions = Permissions::from_bits_truncate(
    Permissions::MUTE_MEMBERS.bits()
        | Permissions::DEAFEN_MEMBERS.bits()
        | Permissions::VIEW_CHANNEL.bits(),
);
