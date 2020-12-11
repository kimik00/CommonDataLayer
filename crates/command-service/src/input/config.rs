use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct KafkaInputConfig {
    #[structopt(long = "kafka-input-group-id", env = "KAFKA_INPUT_GROUP_ID")]
    pub group_id: String,
    #[structopt(long = "kafka-input-brokers", env = "KAFKA_INPUT_BROKERS")]
    /// Comma separated list of brokers (eg. host1:9092,host2:9092)
    pub brokers: String,
    #[structopt(long = "kafka-input-topic", env = "KAFKA_INPUT_TOPIC")]
    pub topic: String,

    #[structopt(
        long = "threaded-task-limit",
        env = "THREADED_TASK_LIMIT",
        default_value = "32"
    )]
    /// Amount of tasks that can be spawned, and process data input, at one given time
    pub task_limit: usize,
}
