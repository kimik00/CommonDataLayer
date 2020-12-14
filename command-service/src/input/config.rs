use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct InputConfig {
    #[structopt(long = "grpc-input-port", env = "GRPC_INPUT_PORT")]
    pub port: u16,
    #[structopt(
        long = "threaded-task-limit",
        env = "THREADED_TASK_LIMIT",
        default_value = "32"
    )]
    /// Amount of tasks that can be spawned, and process data input, at one given time
    pub task_limit: usize,
}
