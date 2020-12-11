use anyhow::Context;
use schema_registry::rpc::schema::schema_registry_client::SchemaRegistryClient;
use serde_json::Value;
use std::path::PathBuf;
use tonic::transport::Channel;

pub async fn connect_to_registry(
    registry_addr: String,
) -> anyhow::Result<SchemaRegistryClient<Channel>> {
    schema_registry::connect_to_registry(registry_addr)
        .await
        .context("Couldn't connect to the schema registry")
}

pub fn read_json(file: Option<PathBuf>) -> anyhow::Result<Value> {
    if let Some(file_name) = file {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(file_name)
            .context("Couldn't open file")?;
        serde_json::from_reader(file).context("File did not contain valid JSON")
    } else {
        serde_json::from_reader(std::io::stdin()).context("File did not contain valid JSON")
    }
}
