use utils::messaging_system::publisher::CommonPublisher;
use uuid::Uuid;

pub use config::ReportServiceConfig;
pub use error::Error;

const APPLICATION_NAME: &str = "Command Service";

mod config;
mod error;

pub struct ReportService {
    producer: CommonPublisher,
    topic: String,
}

impl ReportService {
    pub async fn new(args: ReportServiceConfig) -> Result<Self, Error> {
        Ok(Self {
            producer: CommonPublisher::new_kafka(&args.broker)
                .await
                .map_err(Error::ProducerCreation)?,
            topic: args.topic,
        })
    }

    pub async fn report_failure(
        &self,
        output_plugin: &str,
        description: &str,
        object_id: Uuid,
    ) -> Result<(), Error> {
        let payload = format!(
            r#"{{"application": "{}", "output_plugin": "{}", "description": "{}", "object_id": "{}" }}"#,
            APPLICATION_NAME,
            output_plugin,
            description.replace('"', r#"\""#),
            object_id,
        );

        self.producer
            .publish_message(&self.topic, "command_service.status", payload.into())
            .await
            .map_err(Error::FailedToReport)?;

        Ok(())
    }
}
