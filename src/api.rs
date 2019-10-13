use rocket::{Rocket, State};

use crate::store::{db, new_schema_instance, Schema, Store};

pub fn mount(rocket: Rocket, prefix: &'static str) -> Rocket {
    rocket.attach(new_schema_instance()).mount(
        prefix,
        routes![graphiql, get_graphql_handler, post_graphql_handler],
    )
}

#[rocket::get("/")]
fn graphiql() -> rocket::response::content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    context: db::Connection,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    context: db::Connection,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}
