use schema::{query_client::QueryClient, ObjectIds, SchemaId};

pub mod victoria;

pub mod schema {
    tonic::include_proto!("query");
}

