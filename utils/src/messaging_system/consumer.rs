use anyhow::Context;
use async_stream::try_stream;
use futures_util::stream::{Stream, StreamExt};
use hyper::body::Bytes;
use hyper::Response;
use hyper::StatusCode;
use lapin::{options::BasicConsumeOptions, types::FieldTable};
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    ClientConfig,
};
use std::string::FromUtf8Error;
use std::sync::Arc;
use thiserror::Error as DeriveError;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio_amqp::LapinTokioExt;
use warp::Filter;
use warp::Rejection;

use super::{
    message::CommunicationMessage, message::KafkaCommunicationMessage,
    message::RabbitCommunicationMessage, message::RestCommunicationMessage, CommunicationResult,
};

const REST_DATA_ROUTER_PORT: u16 = 51806;

#[derive(Debug, DeriveError)]
enum RestError {
    #[error("Failed to send data over channel: {0}")]
    ChannelError(SendError<String>),
    #[error("Data was not valid utf-8: {0}")]
    Utf8Error(FromUtf8Error),
}

impl warp::reject::Reject for RestError {}

impl From<RestError> for Rejection {
    fn from(error: RestError) -> Rejection {
        warp::reject::custom(error)
    }
}

pub enum CommonConsumer {
    Kafka {
        consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    },
    RabbitMq {
        consumer: lapin::Consumer,
    },
    Rest {
        message_receiver: UnboundedReceiver<String>,
    },
}
impl CommonConsumer {
    pub async fn new_kafka(
        group_id: &str,
        brokers: &str,
        topics: &[&str],
    ) -> CommunicationResult<CommonConsumer> {
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", &group_id)
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Consumer creation failed")?;

        rdkafka::consumer::Consumer::subscribe(&consumer, topics)
            .context("Can't subscribe to specified topics")?;

        Ok(CommonConsumer::Kafka {
            consumer: Arc::new(consumer),
        })
    }

    pub async fn new_rabbit(
        connection_string: &str,
        consumer_tag: &str,
        queue_name: &str,
    ) -> CommunicationResult<CommonConsumer> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;
        let consumer = channel
            .basic_consume(
                queue_name,
                consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(CommonConsumer::RabbitMq { consumer })
    }

    pub async fn new_rest() -> CommunicationResult<CommonConsumer> {
        async fn handle_response(
            data: Bytes,
            sender: UnboundedSender<String>,
        ) -> Result<Response<String>, Rejection> {
            let body = String::from_utf8(data.to_vec()).map_err(RestError::Utf8Error)?;
            sender.send(body).map_err(RestError::ChannelError)?;

            Ok(Response::builder().body("OK".to_owned()).unwrap())
        }

        let (sender, receiver) = unbounded_channel::<String>();
        let filter = warp::post()
            .and(warp::body::bytes())
            .and(warp::any().map(move || sender.clone()))
            .and_then(handle_response)
            .recover(|error: Rejection| async move {
                if let Some(err) = error.find::<RestError>() {
                    Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(err.to_string())
                        .unwrap())
                } else {
                    Err(error)
                }
            });

        tokio::spawn(warp::serve(filter).run(([0, 0, 0, 0], REST_DATA_ROUTER_PORT)));

        Ok(CommonConsumer::Rest {
            message_receiver: receiver,
        })
    }

    pub async fn consume(
        &mut self,
    ) -> impl Stream<Item = CommunicationResult<Box<dyn CommunicationMessage + '_>>> {
        try_stream! {
            match self {
                CommonConsumer::Kafka { consumer } => {
                    let mut message_stream = consumer.start();
                    while let Some(message) = message_stream.next().await {
                        let message = message?;
                        yield Box::new(KafkaCommunicationMessage{message,consumer:consumer.clone()}) as Box<dyn CommunicationMessage>;
                    }
                }
                CommonConsumer::RabbitMq { consumer } => {
                    while let Some(message) = consumer.next().await {
                        let message = message?;
                        yield Box::new(RabbitCommunicationMessage{channel:message.0, delivery:message.1})as Box<dyn CommunicationMessage>;
                    }
                }
                CommonConsumer::Rest { message_receiver} => {
                    while let Some(message) = message_receiver.next().await {
                        yield Box::new(RestCommunicationMessage { body: message }) as Box<dyn CommunicationMessage>;
                    }
                }
            }
        }
    }

    /// Leaks consumer to guarantee consumer never be dropped.
    /// Static consumer lifetime is required for consumed messages to be passed to spawned futures.
    ///
    /// Use with causion as it can cause memory leaks.
    pub fn leak(self) -> &'static mut CommonConsumer {
        Box::leak(Box::new(self))
    }
}
