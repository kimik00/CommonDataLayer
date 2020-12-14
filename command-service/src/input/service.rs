use crate::communication::{GenericMessage, MessageRouter};
use crate::input::Error;
use crate::output::OutputPlugin;
use log::{error, trace};
use rpc::command_service::command_service_server::{CommandService, CommandServiceServer};
use rpc::command_service::{Empty, InsertMessage};
use std::process;
use tonic::{transport::Server, Request, Response, Status};
use utils::messaging_system::Result;
use utils::metrics::counter;
use utils::task_limiter::TaskLimiter;

pub struct RpcInput<P: OutputPlugin> {
    message_router: MessageRouter<P>,
    task_limiter: TaskLimiter,
}

impl<P: OutputPlugin> RpcInput<P> {
    pub async fn new(message_router: MessageRouter<P>, task_limit: usize) -> Self {
        Self {
            message_router,
            task_limiter: TaskLimiter::new(task_limit),
        }
    }

    async fn handle_message(message: InsertMessage, router: MessageRouter<P>) -> Result<(), Error> {
        counter!("cdl.command-service.input-request", 1);

        let generic_message = Self::build_message(message)?;

        trace!("Received message {:?}", generic_message);

        router
            .handle_message(generic_message)
            .await
            .map_err(Error::CommunicationError)?;

        Ok(())
    }

    fn build_message(message: InsertMessage) -> Result<GenericMessage, Error> {
        Ok(GenericMessage {
            object_id: message.object_id.parse().map_err(Error::KeyNotValidUuid)?,
            schema_id: message.schema_id.parse().map_err(Error::KeyNotValidUuid)?,
            timestamp: message.timestamp,
            payload: message.data,
        })
    }

    pub async fn serve(self, port: u16) -> Result<(), Error> {
        Server::builder()
            .add_service(CommandServiceServer::new(self))
            .serve(([0, 0, 0, 0], port).into())
            .await
            .map_err(Error::FailedToListenToGrpc)
    }
}

#[tonic::async_trait]
impl<P: OutputPlugin> CommandService for RpcInput<P> {
    async fn insert(&self, request: Request<InsertMessage>) -> Result<Response<Empty>, Status> {
        let message = request.into_inner();
        let router = self.message_router.clone();

        self.task_limiter
            .run(async move || {
                if let Err(err) = Self::handle_message(message, router).await {
                    error!("Failed to handle message: {}", err);
                    process::abort();
                }
            })
            .await;

        Ok(Response::new(Empty {}))
    }
}
