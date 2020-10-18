//! The definitions for the graphql api

use anyhow::Context as _;
use async_std::task;
use futures::future::join_all;
use juniper::{Context, FieldResult, RootNode};
use rarity_permission_calculator::Calculator;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    mem::transmute,
    ops::Deref,
    sync::mpsc,
    sync::Arc,
};
use twilight_cache_inmemory::{model::CachedGuild, model::CachedMember, InMemoryCache};
use twilight_gateway::Shard;
use twilight_http::Client;
use twilight_model::{
    channel::{self, permission_overwrite::PermissionOverwriteType, GuildChannel},
    id::{ChannelId, GuildId, UserId},
    user::CurrentUser,
    voice::VoiceState,
};

// FIXME: remove once https://github.com/rarity-rs/permission-calculator upgrades to v 0.2
use rarity_permission_calculator::prelude as model_v1;

use crate::consts::REQUIRED_PERMISSIONS;

#[derive(Debug)]
/// The juniper context to provide access to the discord api and bot
pub struct DiscordContext {
    /// The discord cache connected to the gateway
    pub cache: InMemoryCache,
    /// The shard, connected to the gateway
    pub shard: Shard,
    /// The discord http client for rest calls
    pub http: Client,
}
impl Context for DiscordContext {}

/// A macro to create transparent wrappers of non graphql types for use with juniper
macro_rules! transparent_wrapper {
    (
        $(
            $(#[$outer:meta])*
            pub struct $wrapper:ident($wrapped:ty);
        )*
    ) => {
        $(
            #[derive(Clone, Debug)]
            $(#[$outer])*
            pub struct $wrapper($wrapped);

            impl Deref for $wrapper {
                type Target = $wrapped;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl From<$wrapped> for $wrapper {
                fn from(channel: $wrapped) -> Self {
                    $wrapper(channel)
                }
            }
        )*
    };
}

// Create the wrapper types
transparent_wrapper! {
    /// A voice channel voice state
    pub struct VoiceChannelState(Arc<VoiceState>);
    /// A discord channel category
    pub struct CategoryChannel(channel::CategoryChannel);
    /// A discord voice channel
    pub struct VoiceChannel(channel::VoiceChannel);
    /// A discord guild
    pub struct Guild(Arc<CachedGuild>);
    /// A discord guild member
    pub struct Member(Arc<CachedMember>);
    /// The bots' user
    pub struct Me(Arc<CurrentUser>);
}

// Create try from implementation for subtypes of enums
impl TryFrom<Arc<GuildChannel>> for CategoryChannel {
    type Error = ();

    fn try_from(channel: Arc<GuildChannel>) -> Result<Self, Self::Error> {
        match channel.as_ref() {
            GuildChannel::Category(channel) => Ok(Self(channel.clone())),
            _ => Err(()),
        }
    }
}
impl TryFrom<Arc<GuildChannel>> for VoiceChannel {
    type Error = ();

    fn try_from(channel: Arc<GuildChannel>) -> Result<Self, Self::Error> {
        match channel.as_ref() {
            GuildChannel::Voice(channel) => Ok(Self(channel.clone())),
            _ => Err(()),
        }
    }
}

// Create juniper objects
#[juniper::object(Context = DiscordContext)]
impl Member {
    fn avatar(&self) -> Option<&String> {
        self.user.avatar.as_ref()
    }
    fn joined_at(&self) -> Option<&String> {
        self.joined_at.as_ref()
    }
    fn mute(&self) -> bool {
        self.mute
    }
    fn nick(&self) -> &Option<String> {
        &self.nick
    }
    fn color(&self, discord: &DiscordContext) -> FieldResult<Option<i32>> {
        let mut roles = self
            .roles
            .iter()
            .cloned()
            .filter_map(|role_id| discord.cache.role(role_id))
            .collect::<Vec<_>>();
        roles.sort_by_key(|role| role.position);
        Ok(roles.last().map(|role| role.color.try_into()).transpose()?)
    }
    fn discriminator(&self) -> &str {
        self.user.discriminator.as_str()
    }
    fn id(&self) -> String {
        self.user.id.to_string()
    }
    fn name(&self) -> &str {
        self.user.name.as_str()
    }
    fn bot(&self) -> bool {
        self.user.bot
    }
}

#[juniper::object(Context = DiscordContext)]
impl VoiceChannelState {
    fn id(&self) -> String {
        self.user_id.to_string()
    }
    fn deaf(&self) -> bool {
        self.deaf
    }
    fn mute(&self) -> bool {
        self.mute
    }
    fn self_deaf(&self) -> bool {
        self.self_deaf
    }
    fn self_mute(&self) -> bool {
        self.self_mute
    }
    fn channel_id(&self) -> Option<String> {
        self.channel_id.map(|id| id.to_string())
    }
    fn member(&self, discord: &DiscordContext) -> FieldResult<Member> {
        Ok(discord
            .cache
            .member(
                self.guild_id
                    .context("Voice channel provided was not in a guild")?,
                self.user_id,
            )
            .context("Member does not exist in cache")?
            .into())
    }
}

#[juniper::object]
impl CategoryChannel {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn name(&self) -> &str {
        self.name.as_ref()
    }
    fn position(&self) -> FieldResult<i32> {
        Ok(self.position.try_into()?)
    }
}

#[juniper::object(Context = DiscordContext)]
impl VoiceChannel {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn user_limit(&self) -> FieldResult<Option<i32>> {
        Ok(self.user_limit.map(i32::try_from).transpose()?)
    }
    fn position(&self) -> FieldResult<i32> {
        Ok(self.position.try_into()?)
    }
    fn category(&self, discord: &DiscordContext) -> Option<CategoryChannel> {
        self.parent_id.and_then(|parent_id| {
            discord
                .cache
                .guild_channel(parent_id)
                .and_then(|parent| parent.try_into().ok())
        })
    }

    /// If the bot can operate on the guild
    fn can_operate(&self, discord: &DiscordContext) -> FieldResult<bool> {
        let guild_id = self.guild_id.context("Voice channel missing guild_id")?;
        let guild = discord
            .cache
            .guild(guild_id)
            .context("Voice channel guild does not exist")?;
        let roles = discord
            .cache
            .guild_roles(guild_id)
            .context("Unable to get roles for the guild")?
            .into_iter()
            .map(|role_id| {
                discord
                    .cache
                    .role(role_id)
                    .map(|role| (role_id, role.permissions))
            })
            .collect::<Option<HashMap<_, _>>>()
            .context("Failed to get role information from cache")?;

        let bot_user = discord
            .cache
            .current_user()
            .context("The bot was unable to get information on its user")?;

        let bot_member = discord
            .cache
            .member(guild_id, bot_user.id)
            .context("The bot was unable to get information on itself in the guild")?;

        let permissions = Calculator::new(
            model_v1::GuildId::from(guild_id.0),
            model_v1::UserId::from(guild.owner_id.0),
            &roles
                .into_iter()
                .map(|(role_id, permissions)| {
                    (
                        model_v1::RoleId::from(role_id.0),
                        model_v1::Permissions::from_bits_truncate(permissions.bits()),
                    )
                })
                .collect::<HashMap<_, _>>(),
        )
        .member(
            model_v1::UserId::from(bot_user.id.0),
            bot_member
                .roles
                .iter()
                .map(|role| model_v1::RoleId::from(role.0))
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .in_channel(
            unsafe { transmute(self.kind) },
            self.permission_overwrites
                .iter()
                .map(|perm| model_v1::PermissionOverwrite {
                    allow: model_v1::Permissions::from_bits_truncate(perm.allow.bits()),
                    deny: model_v1::Permissions::from_bits_truncate(perm.deny.bits()),
                    kind: match perm.kind {
                        PermissionOverwriteType::Member(user_id) => {
                            model_v1::PermissionOverwriteType::Member(model_v1::UserId::from(
                                user_id.0,
                            ))
                        }
                        PermissionOverwriteType::Role(role_id) => {
                            model_v1::PermissionOverwriteType::Role(model_v1::RoleId::from(
                                role_id.0,
                            ))
                        }
                    },
                })
                .collect::<Vec<_>>()
                .as_slice(),
        )?;

        Ok(permissions.contains(REQUIRED_PERMISSIONS))
    }

    fn states(&self, discord: &DiscordContext) -> Vec<VoiceChannelState> {
        discord
            .cache
            .voice_channel_states(self.id)
            .unwrap_or_default()
            .into_iter()
            .map(|state| state.into())
            .collect()
    }
}

/// A discord guild
#[juniper::object(Context = DiscordContext)]
impl Guild {
    /// The guilds snowflake id
    fn id(&self) -> String {
        self.id.to_string()
    }
    /// The guild name
    fn name(&self) -> &str {
        self.name.as_str()
    }
    /// Weather or not the guild is unavailable
    fn unavailable(&self) -> bool {
        self.unavailable
    }
    /// The snowflake id of the owner of the guild
    fn owner(&self, discord: &DiscordContext) -> FieldResult<Member> {
        Ok(discord
            .cache
            .member(self.id, self.owner_id)
            .context("The guild owner was not found in the cache")?
            .into())
    }
    /// The icon of the guild
    fn icon(&self) -> Option<&String> {
        self.icon.as_ref()
    }
    /// The banner of the guild
    fn banner(&self) -> Option<&String> {
        self.banner.as_ref()
    }
    /// Get the voice channels in the guild
    fn voice_channels(&self, discord: &DiscordContext) -> Vec<VoiceChannel> {
        discord
            .cache
            .guild_channels(self.id)
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| {
                        discord
                            .cache
                            .guild_channel(id)
                            .and_then(|c| c.try_into().ok())
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
    /// Get a specific voice channel in the guild
    fn voice_channel(
        &self,
        discord: &DiscordContext,
        id: String,
    ) -> FieldResult<Option<VoiceChannel>> {
        Ok(discord
            .cache
            .guild_channel(ChannelId(id.parse().context("Invalid channel id")?))
            .and_then(|c| c.try_into().ok()))
    }
    fn members(&self, discord: &DiscordContext) -> Vec<Member> {
        discord
            .cache
            .guild_members(self.id)
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| {
                        discord
                            .cache
                            .member(self.id, id)
                            .map(|member| member.into())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    fn member(&self, discord: &DiscordContext, id: String) -> FieldResult<Option<Member>> {
        Ok(discord
            .cache
            .member(self.id, UserId(id.parse().context("Invalid user id")?))
            .map(|member| member.into()))
    }
}

#[juniper::object]
impl Me {
    fn name(&self) -> &str {
        &self.name
    }
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn discriminator(&self) -> &str {
        &self.discriminator
    }
}

#[derive(Copy, Clone, Debug)]
/// The root object for GraphQL queries
pub struct QueryRoot;

#[juniper::object(Context = DiscordContext)]
impl QueryRoot {
    /// Get a guild by id
    fn guild(discord: &DiscordContext, id: String) -> FieldResult<Option<Guild>> {
        Ok(discord
            .cache
            .guild(GuildId(id.parse().context("Invalid guild id")?))
            .map(|g| g.into()))
    }
    fn shared_guilds(discord: &DiscordContext, user: String) -> FieldResult<Vec<Guild>> {
        let user = UserId(user.parse().context("Invalid user id")?);

        task::block_on(async move {
            for guild in discord.http.current_user_guilds().limit(100)?.await? {
                discord.http.guild_members(guild.id).await?;
                dbg!(guild);
            }

            Ok(vec![]) // FIXME:
        })

        // Ok(
        //     .into_iter()
        //     .filter(async |guild| {
        //         discord
        //             .http
        //             .guild_members(guild.id)
        //             .await
        //             .context("Failed to get the guild members from a guild")?
        //             .into_iter()
        //             .any(|member| member.user.id == user)
        //     })
        //     .map(|guild| {
        //         discord
        //             .cache
        //             .guild(guild.id)
        //             .context("Guild not found in cache")?
        //             .into()
        //     })
        //     .collect())
    }
    /// Get information about the bot user
    fn bot(&self, discord: &DiscordContext) -> FieldResult<Me> {
        Ok(discord
            .cache
            .current_user()
            .context("Unable to get information on the bot user from the cache")?
            .into())
    }
    // TODO: ME as in logged in user
}

#[derive(Copy, Clone, Debug)]
/// The root object for GraphQL mutations
pub struct MutationRoot;

#[juniper::object(Context = DiscordContext)]
impl MutationRoot {
    fn mute(
        guild_id: String,
        channel_id: String,
        discord: &DiscordContext,
    ) -> FieldResult<Vec<String>> {
        let guild_id = GuildId(guild_id.parse().context("Invalid guild id")?);
        let channel_id = ChannelId(channel_id.parse().context("Invalid channel id")?);

        mass_update_voice_state(discord, channel_id, guild_id, true)
            .map(|ids| ids.into_iter().map(|id| id.to_string()).collect())
    }
    fn unmute(
        guild_id: String,
        channel_id: String,
        discord: &DiscordContext,
    ) -> FieldResult<Vec<String>> {
        let guild_id = GuildId(guild_id.parse().context("Invalid guild id")?);
        let channel_id = ChannelId(channel_id.parse().context("Invalid channel id")?);

        mass_update_voice_state(discord, channel_id, guild_id, false)
            .map(|ids| ids.into_iter().map(|id| id.to_string()).collect())
    }
    // fn create_human(new_human: NewHuman) -> FieldResult<Human> {
    //     Ok(Human {
    //         id: "1234".to_owned(),
    //         name: new_human.name,
    //         appears_in: new_human.appears_in,
    //         home_planet: new_human.home_planet,
    //     })
    // }
}

fn mass_update_voice_state(
    discord: &DiscordContext,
    channel_id: ChannelId,
    guild_id: GuildId,
    mute: bool,
) -> FieldResult<Vec<UserId>> {
    if let Some(states) = discord.cache.voice_channel_states(channel_id) {
        let (send_muted, recieve_muted) = mpsc::channel();

        for chunk in states.chunks(10) {
            task::block_on(join_all(chunk.into_iter().map(|state| {
                let send_muted = send_muted.clone();

                async move {
                    if state.mute != mute {
                        if let Ok(_) = discord
                            .http
                            .update_guild_member(guild_id, state.user_id)
                            .mute(mute)
                            .await
                        {
                            send_muted.send(state.user_id).ok();
                        }
                    }
                }
            })));
        }

        Ok(recieve_muted.try_iter().collect())
    } else {
        Ok(Vec::new())
    }
}

/// The graphql schema described in this file
pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

/// Create the GraphQL schema described in this file
#[must_use = "You need to do something with the schema you have created"]
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot)
}
