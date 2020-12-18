use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::{config::Config, error::Error};
use rpc::query_service::query_service_client::QueryServiceClient;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use tokio::sync::{Mutex, MutexGuard};

pub struct Context {
    config: Arc<Config>,
    registry_conn: Mutex<Option<SchemaRegistryClient<Channel>>>,
    query_router_conn: Mutex<Option<QueryServiceClient<Channel>>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(config: Arc<Config>) -> Self {
        Context {
            config,
            registry_conn: Mutex::new(None),
            query_router_conn: Mutex::new(None),
        }
    }

    pub async fn connect_to_registry(
        &self,
    ) -> Result<Conn<'_, SchemaRegistryClient<Channel>>, Error> {
        let mut conn = self.registry_conn.lock().await;

        if conn.is_none() {
            let new_conn = rpc::schema_registry::connect(self.config.registry_addr.clone()).await?;
            *conn = Some(new_conn);
        }

        Ok(Conn { conn })
    }

    //TODO: Use it. Query router or query service? Decide
    pub async fn connect_to_query_router(
        &self,
    ) -> Result<Conn<'_, QueryServiceClient<Channel>>, Error> {
        let mut conn = self.query_router_conn.lock().await;

        if conn.is_none() {
            let new_conn =
                rpc::query_service::connect(self.config.query_router_addr.clone()).await?;
            *conn = Some(new_conn);
        }

        Ok(Conn { conn })
    }
}

pub struct Conn<'c, C> {
    conn: MutexGuard<'c, Option<C>>,
}

impl<'c, C> Deref for Conn<'c, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().unwrap()
    }
}

impl<'c, C> DerefMut for Conn<'c, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().unwrap()
    }
}
