#![deny(unused_must_use)]
#![feature(decl_macro, proc_macro_hygiene)]

use async_std::task;
use content::Html;
use graphql::{create_schema, Schema};
use juniper_rocket::{graphiql_source, GraphQLRequest, GraphQLResponse};
use log::{debug, error, info, trace, warn};
use rocket::{response::content, routes, State};
use serenity::{
    async_trait, client::Context, client::EventHandler, model::channel::Channel,
    model::channel::ChannelType, model::id::ChannelId, model::prelude::Ready, Client,
};
use std::{env, net::Ipv4Addr, net::SocketAddrV4, sync::Arc};

mod graphql;

const CHANNEL_ID: ChannelId = ChannelId(717435160378867772);

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        task::spawn();
        //     let channel = CHANNEL_ID
        //         .to_channel(ctx)
        //         .await
        //         .expect("The channel stored in CHANNEL_ID does not exist");

        //     if let Channel::Guild(channel) = channel {
        //         if channel.kind == ChannelType::Voice {
        //             println!("{:#?}", channel.bitrate);
        //         } else {
        //             panic!("The channel provided was not a guild voice channel, check the channel id");
        //         }
        //     } else {
        //         panic!("The channel provided was not a guild channel, check the channel id");
        //     }
    }
}

#[rocket::get("/")]
fn graphiql() -> Html<String> {
    graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    // context: State<Database>,
    request: GraphQLRequest,
    schema: State<Schema>,
) -> GraphQLResponse {
    request.execute(&schema, &())
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    // context: State<Database>,
    request: GraphQLRequest,
    schema: State<Schema>,
) -> GraphQLResponse {
    request.execute(&schema, &())
}

#[async_std::main]
async fn main() {
    env::set_var("RUST_LOG", "_=info");
    pretty_env_logger::init();

    let token =
        env::var("DISCORD_TOKEN").expect("You must provide a DISCORD_TOKEN environment variable");

    let http = Client::new(&token);

    let mut shard = ShardBuilder::new(token).http_client(http).intents(Some(
        GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
    ));
    shard.start().await?;

    let cache = InMemoryCacheBuilder::new()
        .event_types(
            EventType::VOICE_STATE_UPDATE | EventType::GUILD_UPDATE | EventType::GUILD_CREATE,
        )
        .build();

    // Startup an event loop for each event in the event stream
    {
        let shard = shard.clone();
        task::spawn(async move {
            let mut events = shard.events();

            while let Some(event) = events.next().await {
                cache.update(&event);

                println!("{:?}", event);
            }
        });
    }

    rocket::ignite()
        // .manage(Database::new())
        .manage(shard)
        .manage(create_schema())
        .mount(
            "/",
            routes![
                crate::graphiql,
                crate::get_graphql_handler,
                crate::post_graphql_handler
            ],
        )
        .launch()
    // ctx.lock().await.shutdown_all();
}
