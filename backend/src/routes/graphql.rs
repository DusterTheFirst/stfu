//! The graphql based routes and handlers

#![allow(clippy::needless_pass_by_value, clippy::must_use_candidate)]

use crate::{
    auth::OauthUser,
    graphql::{DiscordContext, GraphQLContext, Schema},
};
use juniper_rocket_async::{graphiql_source, GraphQLRequest, GraphQLResponse};
use rocket::{
    http::Status,
    response::{content::Html, Redirect},
    uri, State,
};

/// The graphiql IDE
#[rocket::get("/")]
pub fn graphiql(_user: OauthUser) -> Html<String> {
    graphiql_source("/graphql")
}

/// A redirect to the auth if not logged in
#[rocket::get("/", rank = 1)]
pub fn graphiql_no_auth() -> Redirect {
    Redirect::to(uri!(super::auth::oauth_login: "/"))
}

/// The get based graphql handler
#[rocket::get("/graphql?<request>")]
pub async fn get_graphql_handler<'r>(
    discord: State<DiscordContext, 'r>,
    oauth: OauthUser,
    schema: State<Schema, 'r>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request
        .execute(
            &schema,
            &GraphQLContext {
                discord: discord.clone(),
                user: oauth,
            },
        )
        .await
}

/// An error code if not logged in
#[rocket::get("/graphql?<_request>", rank = 1)]
pub fn get_graphql_no_auth(_request: GraphQLRequest) -> Status {
    Status::Unauthorized
}

/// The post based graphql handler
#[rocket::post("/graphql", data = "<request>")]
pub async fn post_graphql_handler<'r>(
    discord: State<DiscordContext, 'r>,
    oauth: OauthUser,
    schema: State<Schema, 'r>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request
        .execute(
            &schema,
            &GraphQLContext {
                discord: discord.clone(),
                user: oauth,
            },
        )
        .await
}

/// An error code if not logged in
#[rocket::post("/graphql", data = "<_request>", rank = 1)]
pub fn post_graphql_no_auth(_request: GraphQLRequest) -> Status {
    Status::Unauthorized
}
