use schema::{query_client::QueryClient, Range, Tag};
use tonic::transport::Channel;

pub mod victoria;

pub mod schema {
    tonic::include_proto!("query");
}
