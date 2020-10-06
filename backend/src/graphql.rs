use juniper::FieldResult;
use juniper::RootNode;
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use serenity::model::id::{ChannelId, GuildId};

/// A discord voice channel
struct VoiceChannel {
    members: Vec</* VoiceChannelMember */ String>,
    name: String,
    id: ChannelId,
}

#[juniper::object]
impl VoiceChannel {
    fn members(&self) -> &[String] {
        self.members.as_slice()
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn id(&self) -> String {
        self.id.to_string()
    }
}

/// A discord guild
struct Guild {
    id: GuildId,
}

/// A discord guild
#[juniper::object]
impl Guild {
    fn id(&self) -> String {
        self.id.to_string()
    }
    fn voice_channel(id: String) -> FieldResult<VoiceChannel> {
        let id = ChannelId::from(id.parse::<u64>()?);

        Ok(VoiceChannel {
            id,
            members: vec![],
            name: "test".into(),
        })
    }
}

pub struct QueryRoot;

#[juniper::object]
impl QueryRoot {
    fn guild(id: String) -> FieldResult<Guild> {
        let id = GuildId::from(id.parse::<u64>()?);

        Ok(Guild { id })
    }
}

pub struct MutationRoot;

#[juniper::object]
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
