use futures_util::stream::StreamExt;
use log::error;
use tokio::pin;

use crate::communication::{GenericMessage, MessageRouter};
use crate::input::{Error, KafkaInputConfig};
use utils::{
    message_types::CommandServiceInsertMessage,
    messaging_system::{
        consumer::CommonConsumer, message::CommunicationMessage, CommunicationResult,
    },
    metrics::counter,
    task_limiter::TaskLimiter,
};

pub struct KafkaInput {
    consumer: CommonConsumer,
    message_router: MessageRouter,
    task_limiter: TaskLimiter,
}

impl KafkaInput {
    pub async fn new(
        config: KafkaInputConfig,
        message_router: MessageRouter,
    ) -> Result<Self, Error> {
        let consumer =
            CommonConsumer::new_kafka(&config.group_id, &config.brokers, &[&config.topic])
                .await
                .map_err(Error::ConsumerCreationFailed)?;
        Ok(Self {
            consumer,
            message_router,
            task_limiter: TaskLimiter::new(config.task_limit),
        })
    }

    async fn handle_message(
        router: MessageRouter,
        message: CommunicationResult<Box<dyn CommunicationMessage>>,
    ) -> Result<(), Error> {
        counter!("cdl.command-service.input-request", 1);
        let message = message.map_err(Error::FailedReadingMessage)?;

        let generic_message = Self::build_message(message.as_ref())?;

        router
            .handle_message(generic_message)
            .await
            .map_err(Error::CommunicationError)?;

        Ok(())
    }

    fn build_message(message: &'_ dyn CommunicationMessage) -> Result<GenericMessage, Error> {
        let json = message.payload().map_err(Error::MissingPayload)?;
        let event: CommandServiceInsertMessage =
            serde_json::from_str(json).map_err(Error::PayloadDeserializationFailed)?;

        Ok(GenericMessage {
            object_id: event.object_id,
            schema_id: event.schema_id,
            timestamp: event.timestamp,
            payload: event.payload.to_string().as_bytes().to_vec(),
        })
    }

    pub async fn listen(self) -> Result<(), Error> {
        let consumer = self.consumer.leak();
        let message_stream = consumer.consume().await;
        pin!(message_stream);

        while let Some(message) = message_stream.next().await {
            let router = self.message_router.clone();

            self.task_limiter
                .run(async move || {
                    if let Err(err) = Self::handle_message(router, message).await {
                        error!("Failed to handle message: {}", err);
                    }
                })
                .await;
        }

        Ok(())
    }
}
