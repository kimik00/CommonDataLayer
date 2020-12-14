use anyhow::Context;
use log::error;
use lru_cache::LruCache;
use rpc::{command_service::InsertMessage, schema_registry::Id};
use serde::{Deserialize, Serialize};
use std::{
    process,
    sync::{Arc, Mutex},
};
use structopt::StructOpt;
use tokio::pin;
use tokio::stream::StreamExt;
use utils::{
    abort_on_poison,
    message_types::DataRouterInputData,
    messaging_system::{
        consumer::CommonConsumer, message::CommunicationMessage, publisher::CommonPublisher,
    },
    metrics::{self, counter},
};
use uuid::Uuid;

const SERVICE_NAME: &str = "data-router";

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    #[structopt(long, env)]
    pub kafka_group_id: String,
    #[structopt(long, env)]
    pub kafka_topic: String,
    #[structopt(long, env)]
    pub kafka_brokers: String,
    #[structopt(long, env)]
    pub kafka_error_channel: String,
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    #[structopt(long, env)]
    pub cache_capacity: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config: Config = Config::from_args();
    metrics::serve();

    let consumer = CommonConsumer::new_kafka(
        &config.kafka_group_id,
        &config.kafka_brokers,
        &[&config.kafka_topic],
    )
    .await?;
    let producer = Arc::new(CommonPublisher::new_kafka(&config.kafka_brokers).await?);
    let cache = Arc::new(Mutex::new(LruCache::new(config.cache_capacity)));
    let consumer = consumer.leak();
    let message_stream = consumer.consume().await;
    pin!(message_stream);

    let kafka_error_channel = Arc::new(config.kafka_error_channel);
    let schema_registry_addr = Arc::new(config.schema_registry_addr);

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(message) => {
                tokio::spawn(handle_message(
                    message,
                    producer.clone(),
                    cache.clone(),
                    kafka_error_channel.clone(),
                    schema_registry_addr.clone(),
                ));
            }
            Err(error) => {
                error!("Error fetching data from message queue {:?}", error);
                // no error handling necessary - message won't be acked - it was never delivered properly
            }
        };
    }
    Ok(())
}

async fn handle_message(
    message: Box<dyn CommunicationMessage>,
    producer: Arc<CommonPublisher>,
    cache: Arc<Mutex<LruCache<Uuid, String>>>,
    kafka_error_channel: Arc<String>,
    schema_registry_addr: Arc<String>,
) {
    let result: anyhow::Result<()> = async {
        let event: DataRouterInputData =
            serde_json::from_str(message.payload()?).context("Payload deserialization failed")?;
        let insert_address =
            get_schema_insert_address(&cache, &event, &schema_registry_addr).await?;

        let mut client = rpc::command_service::connect(insert_address).await?;
        client
            .insert(InsertMessage {
                object_id: event.object_id.to_string(),
                schema_id: event.schema_id.to_string(),
                timestamp: message.timestamp()?,
                data: serde_json::to_vec(event.data).context("unable to serialize event")?,
            })
            .await?;

        Ok(())
    }
    .await;

    counter!("cdl.data-router.input-request", 1);

    if let Err(error) = result {
        error!("{:?}", error);
        send_message(
            producer.as_ref(),
            &kafka_error_channel,
            SERVICE_NAME,
            format!("{:?}", error).into(),
        )
        .await;
    }
    let ack_result = message.ack().await;
    if let Err(e) = ack_result {
        error!(
            "Fatal error, delivery status for message not received. {:?}",
            e
        );
        process::abort();
    }
}

async fn get_schema_insert_address(
    cache: &Mutex<LruCache<Uuid, String>>,
    event: &DataRouterInputData<'_>,
    schema_addr: &str,
) -> anyhow::Result<String> {
    let recv_channel = cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .get_mut(&event.schema_id)
        .cloned();
    if let Some(val) = recv_channel {
        return Ok(val);
    }

    let mut client = rpc::schema_registry::connect(schema_addr.to_owned()).await?;
    let address = client
        .get_schema_insert_address(Id {
            id: event.schema_id.to_string(),
        })
        .await?
        .into_inner()
        .address;
    cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .insert(event.schema_id, address.clone());

    Ok(address)
}

async fn send_message(producer: &CommonPublisher, topic_name: &str, key: &str, payload: Vec<u8>) {
    let payload_len = payload.len();
    let delivery_status = producer.publish_message(&topic_name, key, payload).await;

    if delivery_status.is_err() {
        error!(
            "Fatal error, delivery status for message not received.  Topic: `{}`, Key: `{}`, Payload len: `{}`, {:?}",
            topic_name, key, payload_len, delivery_status
        );
        process::abort();
    };
}
