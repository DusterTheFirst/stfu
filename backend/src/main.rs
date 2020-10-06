#![deny(unused_must_use)]
#![feature(decl_macro, proc_macro_hygiene)]

use graphql::{create_schema, Schema};
use juniper::{graphiql::graphiql_source};
use juniper_rocket::{GraphQLRequest, GraphQLResponse};
use rocket::{State, response::content};
use serenity::{
    async_trait, client::Context, client::EventHandler, model::channel::Channel,
    model::channel::ChannelType, model::id::ChannelId, model::prelude::Ready, Client,
};
use std::{env, net::Ipv4Addr, net::SocketAddrV4, sync::Arc};
use log::{error, warn, trace, debug, info};

mod graphql;

const CHANNEL_ID: ChannelId = ChannelId(717435160378867772);

struct Handler;

// #[async_trait]
impl EventHandler for Handler {
    // async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
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
    // }
}

// #[async_std::main]
// async fn main() {
    // let token =
    //     env::var("DISCORD_TOKEN").expect("You must provide a DISCORD_TOKEN environment variable");

    // let mut client = Client::new(token)
    //     .event_handler(Handler)
    //     .await
    //     .expect("Error creating client");

    // if let Err(why) = client.start().await {
    //     println!("An error occurred while running the client: {:?}", why);
    // }

    // ctx.lock().await.shutdown_all();
// }

#[rocket::get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
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

fn main() {
    // env::set_var("RUST_LOG", "_");
    pretty_env_logger::init();

    rocket::ignite()
        // .manage(Database::new())
        .manage(create_schema())
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch();
}