//! The definitions for the graphql api

use anyhow::Context as _;
use futures::future::join_all;
use juniper::{graphql_object, Context, EmptySubscription, FieldResult, RootNode};
use std::{
    collections::HashMap,
    collections::HashSet,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    mem::transmute,
    ops::Deref,
    sync::mpsc,
    sync::Arc,
};
use twilight_cache_inmemory::{model::CachedGuild, model::CachedMember, InMemoryCache};
use twilight_gateway::Shard;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::{self, permission_overwrite::PermissionOverwriteType, GuildChannel},
    id::{ChannelId, GuildId, UserId},
    user,
    voice::VoiceState,
};
use twilight_oauth2::Client as OauthClient;
use twilight_permission_calculator::Calculator;

// FIXME: remove once https://github.com/rarity-rs/permission-calculator upgrades to v 0.2
use twilight_permission_calculator::prelude as model_v1;

use crate::{auth::OauthUser, consts::REQUIRED_PERMISSIONS};

/// The juniper context to provide access to the user and discord api
#[derive(Debug)]
pub struct GraphQLContext {
    /// The nested discord context
    pub discord: DiscordContext,
    /// The user who is authenticated with oauth
    pub user: OauthUser,
}
impl Context for GraphQLContext {}

#[derive(Debug, Clone)]
/// The juniper context to provide access to the discord api and bot
///
/// This context derives clone since it is just 4 pointers, it can be cloned rather effortlessly
pub struct DiscordContext {
    /// The discord cache connected to the gateway
    pub cache: InMemoryCache,
    /// The shard, connected to the gateway
    pub shard: Shard,
    /// The discord http client for rest calls
    pub http: HttpClient,
    /// The discord oauth client for authentication
    pub oauth: Arc<OauthClient>,
}

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
                fn from(w: $wrapped) -> Self {
                    $wrapper(w)
                }
            }
        )*
    };
    (
        $(
            $(#[$outer:meta])*
            pub struct $wrapper:ident($wrapped:ty);
            use enum type $enum_name:ident::$variant:ident($var_wrapped:ty);
        )*
    ) => {
        $(
            #[derive(Clone, Debug)]
            $(#[$outer])*
            pub struct $wrapper($wrapped);

            impl Deref for $wrapper {
                type Target = $var_wrapped;

                fn deref(&self) -> &Self::Target {
                    if let $enum_name::$variant(w) = self.0.as_ref() {
                        w
                    } else {
                        unreachable!()
                    }
                }
            }
            impl TryFrom<$wrapped> for $wrapper {
                type Error = ();

                fn try_from(w: $wrapped) -> Result<Self, Self::Error> {
                    if let $enum_name::$variant(_) = w.as_ref() {
                        Ok(Self(w))
                    } else {
                        Err(())
                    }
                }
            }
        )*
    };
}

// Create the wrapper types
transparent_wrapper! {
    /// A voice channel voice state.
    pub struct VoiceChannelState(Arc<VoiceState>);
    /// A discord guild.
    pub struct Guild(Arc<CachedGuild>);
    /// A discord guild member.
    pub struct Member(Arc<CachedMember>);
    /// A current user, either the bot or an oauth user.
    pub struct CurrentUser(Arc<user::CurrentUser>);
}

// Create the wrapper types around enum variants
transparent_wrapper! {
    /// A discord channel category.
    pub struct CategoryChannel(Arc<GuildChannel>);
    use enum type GuildChannel::Category(channel::CategoryChannel);
    /// A discord voice channel.
    pub struct VoiceChannel(Arc<GuildChannel>);
    use enum type GuildChannel::Voice(channel::VoiceChannel);
}

// Create juniper objects

#[graphql_object(Context = GraphQLContext)]
/// A member of a guild.
impl Member {
    /// Avatar hash of the member. FIXME:
    pub fn avatar(&self) -> Option<&String> {
        self.user.avatar.as_ref()
    }

    /// Time the member joined the guild.
    fn joined_at(&self) -> Option<&String> {
        self.joined_at.as_ref()
    }

    /// Member's server mute status.
    fn mute(&self) -> bool {
        self.mute
    }

    /// Member's server deafened status.
    fn deaf(&self) -> bool {
        self.deaf
    }

    /// Member's nickname.
    fn nick(&self) -> &Option<String> {
        &self.nick
    }

    /// Member's color, calculated from their highest, colored, role.
    fn color(&self, context: &GraphQLContext) -> FieldResult<Option<i32>> {
        let mut roles = self
            .roles
            .iter()
            .cloned()
            .filter_map(|role_id| context.discord.cache.role(role_id))
            .collect::<Vec<_>>();

        roles.sort_by_key(|role| role.position);

        Ok(roles.last().map(|role| role.color.try_into()).transpose()?)
    }

    /// Member's discriminator.
    fn discriminator(&self) -> &str {
        self.user.discriminator.as_str()
    }

    /// Member's unique identifier.
    fn id(&self) -> String {
        self.user.id.to_string()
    }

    /// Member's discord username.
    fn name(&self) -> &str {
        self.user.name.as_str()
    }

    /// Member's bot status.
    fn bot(&self) -> bool {
        self.user.bot
    }
}

/// State of a member in a voice channel.
#[graphql_object(Context = GraphQLContext)]
impl VoiceChannelState {
    /// Id of the member who this voice state is about.
    fn id(&self) -> String {
        self.user_id.to_string()
    }

    /// Server deafened status of the member.
    fn deaf(&self) -> bool {
        self.deaf
    }

    /// Server mute status of the member.
    fn mute(&self) -> bool {
        self.mute
    }

    /// Self deafened status of the member.
    fn self_deaf(&self) -> bool {
        self.self_deaf
    }

    /// Self muted status of the member.
    fn self_mute(&self) -> bool {
        self.self_mute
    }

    /// Channel id that this voice state is in.
    fn channel_id(&self) -> Option<String> {
        self.channel_id.map(|id| id.to_string())
    }

    /// Member object associated with the voice state.
    fn member(&self, context: &GraphQLContext) -> FieldResult<Member> {
        Ok(context
            .discord
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

/// A channel category for grouping channels.
#[graphql_object]
impl CategoryChannel {
    /// Id of the category.
    fn id(&self) -> String {
        self.id.to_string()
    }

    /// Name of the category.
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Relative position of the category.
    fn position(&self) -> FieldResult<i32> {
        Ok(self.position.try_into()?)
    }
}

/// A voice channel in a guild.
#[graphql_object(Context = GraphQLContext)]
impl VoiceChannel {
    /// Name of the voice channel.
    fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Unique id of the voice channel.
    fn id(&self) -> String {
        self.id.to_string()
    }

    /// Maximum amount of users allowed in a channel.
    fn user_limit(&self) -> FieldResult<Option<i32>> {
        Ok(self.user_limit.map(i32::try_from).transpose()?)
    }

    /// Relative position of the voice channel.
    fn position(&self) -> FieldResult<i32> {
        Ok(self.position.try_into()?)
    }

    /// The parent channel category
    fn category(&self, context: &GraphQLContext) -> Option<CategoryChannel> {
        self.parent_id.and_then(|parent_id| {
            context
                .discord
                .cache
                .guild_channel(parent_id)
                .and_then(|parent| parent.try_into().ok())
        })
    }

    /// If the bot can operate on the guild.
    fn can_operate(&self, context: &GraphQLContext) -> FieldResult<bool> {
        let guild_id = self.guild_id.context("Voice channel missing guild_id")?;
        let guild = context
            .discord
            .cache
            .guild(guild_id)
            .context("Voice channel guild does not exist")?;
        let roles = context
            .discord
            .cache
            .guild_roles(guild_id)
            .context("Unable to get roles for the guild")?
            .into_iter()
            .map(|role_id| {
                context
                    .discord
                    .cache
                    .role(role_id)
                    .map(|role| (role_id, role.permissions))
            })
            .collect::<Option<HashMap<_, _>>>()
            .context("Failed to get role information from cache")?;

        let bot_user = context
            .discord
            .cache
            .current_user()
            .context("The bot was unable to get information on its user")?;

        let bot_member = context
            .discord
            .cache
            .member(guild_id, bot_user.id)
            .context("The bot was unable to get information on itself in the guild")?;

        // FIXME: On release of v2 to stable
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

    /// Voice channel states in this voice channel.
    fn states(&self, context: &GraphQLContext) -> Vec<VoiceChannelState> {
        context
            .discord
            .cache
            .voice_channel_states(self.id)
            .unwrap_or_default()
            .into_iter()
            .map(|state| state.into())
            .collect()
    }
}

/// A discord guild.
#[graphql_object(Context = GraphQLContext)]
impl Guild {
    /// Guild's snowflake id.
    fn id(&self) -> String {
        self.id.to_string()
    }
    /// Guild's name.
    fn name(&self) -> &str {
        self.name.as_str()
    }
    /// Weather or not the guild is unavailable.
    fn unavailable(&self) -> bool {
        self.unavailable
    }

    /// Guild member object of the owner of the guild.
    fn owner(&self, context: &GraphQLContext) -> FieldResult<Member> {
        Ok(context
            .discord
            .cache
            .member(self.id, self.owner_id)
            .context("The guild owner was not found in the cache")?
            .into())
    }

    /// Icon hash of the guild.
    fn icon(&self) -> Option<&String> {
        self.icon.as_ref()
    }
    /// Banner hash of the guild.
    fn banner(&self) -> Option<&String> {
        self.banner.as_ref()
    }

    /// Voice channels in the guild.
    fn voice_channels(&self, context: &GraphQLContext) -> Vec<VoiceChannel> {
        context
            .discord
            .cache
            .guild_channels(self.id)
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| {
                        context
                            .discord
                            .cache
                            .guild_channel(id)
                            .and_then(|c| c.try_into().ok())
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// A specific voice channel in the guild.
    #[graphql(arguments(id(description = "Id of the voice channel to fetch")))]
    fn voice_channel(
        &self,
        context: &GraphQLContext,
        id: String,
    ) -> FieldResult<Option<VoiceChannel>> {
        Ok(context
            .discord
            .cache
            .guild_channel(ChannelId(id.parse().context("Invalid channel id")?))
            .and_then(|c| c.try_into().ok()))
    }

    /// All the members in the guild.
    fn members(&self, context: &GraphQLContext) -> Vec<Member> {
        context
            .discord
            .cache
            .guild_members(self.id)
            .map(|ids| {
                ids.into_iter()
                    .filter_map(|id| {
                        context
                            .discord
                            .cache
                            .member(self.id, id)
                            .map(|member| member.into())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// A specific member in the guild.
    fn member(&self, context: &GraphQLContext, id: String) -> FieldResult<Option<Member>> {
        Ok(context
            .discord
            .cache
            .member(self.id, UserId(id.parse().context("Invalid user id")?))
            .map(|member| member.into()))
    }

    /// The current logged in user as a member of the guild.
    async fn me(&self, context: &GraphQLContext) -> FieldResult<Member> {
        Ok(context
            .discord
            .cache
            .member(self.id, context.user.http.current_user().await?.id)
            .map(|member| member.into())
            .context("Failed to lookup current user in cache")?)
    }
}

/// A current user object, different from a member since it is detached from a guild.
#[graphql_object]
impl CurrentUser {
    /// Discord username of the user.
    fn name(&self) -> &str {
        &self.name
    }

    /// Unique identifying id of the user.
    fn id(&self) -> String {
        self.id.to_string()
    }

    /// Discriminator of the user.
    fn discriminator(&self) -> &str {
        &self.discriminator
    }

    /// If the user has multi-factor authentication enabled
    fn mfa(&self) -> bool {
        self.mfa_enabled
    }

    /// User's avatar hash.
    fn avatar(&self) -> Option<&String> {
        self.avatar.as_ref()
    }
}

#[derive(Copy, Clone, Debug)]
/// The root object for `GraphQL` queries.
pub struct QueryRoot;

/// The root object for GraphQL queries.
#[graphql_object(Context = GraphQLContext)]
impl QueryRoot {
    /// Get a guild by id.
    #[graphql(arguments(id(description = "Id of the guild to fetch")))]
    fn guild(context: &GraphQLContext, id: String) -> FieldResult<Option<Guild>> {
        Ok(context
            .discord
            .cache
            .guild(GuildId(id.parse().context("Invalid guild id")?))
            .map(|g| g.into()))
    }

    /// Get the intersection of guilds between the logged in user and the bot.
    async fn shared_guilds(context: &GraphQLContext) -> FieldResult<Vec<Guild>> {
        let bot_guilds = context
            .discord
            .http
            .current_user_guilds()
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect::<HashSet<_>>();

        let user_guilds = context
            .user
            .http
            .current_user_guilds()
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect::<HashSet<_>>();

        let intersection = bot_guilds.intersection(&user_guilds).cloned();

        Ok(intersection
            .filter_map(|id| context.discord.cache.guild(id).map(|guild| guild.into()))
            .collect())
    }

    /// Get information about the bot user.
    fn bot(&self, context: &GraphQLContext) -> FieldResult<CurrentUser> {
        Ok(context
            .discord
            .cache
            .current_user()
            .context("Unable to get information on the bot user from the cache")?
            .into())
    }

    /// Get information about the logged in oauth user.
    async fn me(&self, context: &GraphQLContext) -> FieldResult<CurrentUser> {
        Ok(Arc::new(
            context
                .user
                .http
                .current_user()
                .await
                .context("Unable to get information on the current oauth user")?,
        )
        .into())
    }
}

#[derive(Copy, Clone, Debug)]
/// The root object for `GraphQL` mutations.
pub struct MutationRoot;

/// The root object for GraphQL mutations.
#[graphql_object(Context = GraphQLContext)]
impl MutationRoot {
    /// Mute all users in a voice channel.
    ///
    /// # Returns
    /// Id's of users who were successfully muted
    #[graphql(arguments(
        guild_id(description = "Id of the guild that the channel resides in",),
        channel_id(description = "Id of the channel to mutate",),
    ))]
    async fn mute(
        guild_id: String,
        channel_id: String,
        context: &GraphQLContext,
    ) -> FieldResult<Vec<String>> {
        let guild_id = GuildId(guild_id.parse().context("Invalid guild id")?);
        let channel_id = ChannelId(channel_id.parse().context("Invalid channel id")?);

        mass_update_voice_state(context, channel_id, guild_id, true)
            .await
            .map(|ids| ids.into_iter().map(|id| id.to_string()).collect())
    }

    /// Unmute all users in a voice channel.
    ///
    /// # Returns
    /// Id's of users who were successfully un-muted
    #[graphql(arguments(
        guild_id(description = "Id of the guild that the channel resides in",),
        channel_id(description = "Id of the channel to mutate",),
    ))]
    async fn unmute(
        guild_id: String,
        channel_id: String,
        context: &GraphQLContext,
    ) -> FieldResult<Vec<String>> {
        let guild_id = GuildId(guild_id.parse().context("Invalid guild id")?);
        let channel_id = ChannelId(channel_id.parse().context("Invalid channel id")?);

        mass_update_voice_state(context, channel_id, guild_id, false)
            .await
            .map(|ids| ids.into_iter().map(|id| id.to_string()).collect())
    }
}

async fn mass_update_voice_state(
    context: &GraphQLContext,
    channel_id: ChannelId,
    guild_id: GuildId,
    mute: bool,
) -> FieldResult<Vec<UserId>> {
    if let Some(states) = context.discord.cache.voice_channel_states(channel_id) {
        let (send_muted, receive_muted) = mpsc::channel();

        for chunk in states.chunks(10) {
            let send_muted = send_muted.clone();

            join_all(chunk.iter().map(move |state| {
                let send_muted = send_muted.clone();

                async move {
                    if state.mute != mute
                        && context
                            .discord
                            .http
                            .update_guild_member(guild_id, state.user_id)
                            .mute(mute)
                            .await
                            .is_ok()
                    {
                        send_muted.send(state.user_id).ok();
                    }
                }
            }))
            .await;
        }

        Ok(receive_muted.try_iter().collect())
    } else {
        Ok(Vec::new())
    }
}

/// The graphql schema described in this file
pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<GraphQLContext>>;

/// Create the `GraphQL` schema described in this file
#[must_use = "You need to do something with the schema you have created"]
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
