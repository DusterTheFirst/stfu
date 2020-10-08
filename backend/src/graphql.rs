use std::ops::Deref;

use anyhow::Context as _;
use juniper::{Context, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, RootNode};
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::{
    channel::permission_overwrite::PermissionOverwrite,
    channel::permission_overwrite::PermissionOverwriteType,
    channel::GuildChannel,
    guild::Permissions,
    id::{ChannelId, GuildId, UserId},
};

use crate::consts::REQUIRED_PERMISSIONS;

pub struct Discord(InMemoryCache);
impl Discord {
    pub fn wrap(cache: InMemoryCache) -> Self {
        Discord(cache)
    }
}
impl Deref for Discord {
    type Target = InMemoryCache;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Context for Discord {}

/// A discord voice channel
#[derive(Clone, Debug)]
struct VoiceChannel {
    guild: Guild,
    name: String,
    id: ChannelId,
    permission_overwrites: Vec<PermissionOverwrite>,
    parent_id: Option<ChannelId>,
    user_limit: Option<u64>,
    position: i64,
    // FIXME:
}

#[juniper::object(Context = Discord)]
impl VoiceChannel {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn id(&self) -> String {
        self.id.to_string()
    }
    // TODO:

    /// If the bot can operate on the guild
    fn can_operate(&self, context: &Discord) -> FieldResult<bool> {
        let mut permissions = self.guild.permissions.unwrap_or(Permissions::empty());

        let bot_user = context
            .current_user()
            .context("The bot was unable to get information on its user")?;

        let bot_member = context
            .member(self.guild.id, bot_user.id)
            .context("The bot was unable to get information on itself in the guild")?;

        for overwrite in &self.permission_overwrites {
            match overwrite.kind {
                PermissionOverwriteType::Role(id) if bot_member.roles.contains(&id) => {
                    permissions.set(overwrite.allow, true);
                    permissions.set(overwrite.deny, false);
                }
                PermissionOverwriteType::Member(id) if id == bot_user.id => {
                    permissions.set(overwrite.allow, true);
                    permissions.set(overwrite.deny, false);
                }
                _ => {}
            }
        }

        Ok(permissions.contains(REQUIRED_PERMISSIONS))
    }

    // fn members(&self) -> &[String] {
    //     self.members.as_slice()
    // }
}

/// A discord guild
#[derive(Clone, Debug)]
struct Guild {
    /// The guilds snowflake id
    pub id: GuildId,
    // The guilds name
    pub name: String,
    /// Weather or not the guild is unavailable
    pub unavailable: bool,
    /// The snowflake id of the owner of the guild
    pub owner_id: UserId,
    /// The icon of the guild
    pub icon: Option<String>,
    /// The banner of the guild
    pub banner: Option<String>,
    /// The permissions of the guild
    pub permissions: Option<Permissions>,
}

/// A discord guild
#[juniper::object(Context = Discord)]
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
    fn owner_id(&self) -> String {
        self.owner_id.to_string()
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
    fn voice_channels(&self, context: &Discord) -> Option<Vec<VoiceChannel>> {
        context.guild_channels(self.id).map(|ids| {
            ids.into_iter()
                .filter_map(|id| {
                    context
                        .guild_channel(id)
                        .map(|c| match c.as_ref() {
                            GuildChannel::Voice(c) => Some(VoiceChannel {
                                guild: self.clone(),
                                id,
                                name: c.name.clone(),
                                permission_overwrites: c.permission_overwrites.clone(),
                                user_limit: c.user_limit,
                                parent_id: c.parent_id,
                                position: c.position,
                            }),
                            _ => None,
                        })
                        .flatten()
                })
                .collect::<Vec<_>>()
        })
    }
}

/// The bots' user
#[derive(GraphQLObject, Clone, Debug)]
struct Me {}

pub struct QueryRoot;

#[juniper::object(Context = Discord)]
impl QueryRoot {
    fn guild(context: &Discord, id: String) -> FieldResult<Option<Guild>> {
        let id = GuildId::from(id.parse::<u64>()?);

        Ok(context.guild(id).map(|guild| Guild {
            id,
            unavailable: guild.unavailable,
            owner_id: guild.owner_id,
            icon: guild.icon.clone(),
            banner: guild.banner.clone(),
            name: guild.name.clone(),
            permissions: guild.permissions,
        }))
    }
    fn me(&self, context: &Discord) -> Option<Me> {
        context.current_user();
        todo!()
    }
}

pub struct MutationRoot;

#[juniper::object(Context = Discord)]
impl MutationRoot {
    // fn create_human(new_human: NewHuman) -> FieldResult<Human> {
    //     Ok(Human {
    //         id: "1234".to_owned(),
    //         name: new_human.name,
    //         appears_in: new_human.appears_in,
    //         home_planet: new_human.home_planet,
    //     })
    // }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot)
}
