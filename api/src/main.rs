pub mod config;
pub mod context;
pub mod error;
pub mod schema;

use std::sync::Arc;

use config::Config;
use context::Context;
use warp::{http::Response, Filter};

#[tokio::main]
async fn main() {
    let config = Arc::new(Config::from_args());

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });

    let state = warp::any().map(move || Context::new(config.clone()));
    let graphql_filter = juniper_warp::make_graphql_filter(crate::schema::schema(), state.boxed());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter)),
    )
    .run(([0, 0, 0, 0], config.input_port))
    .await
}
