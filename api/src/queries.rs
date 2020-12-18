use crate::context::Context;
use crate::error::Result;
use crate::schema::*;
use futures::stream::{StreamExt, TryStreamExt};
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};
use rpc::schema_registry::Empty;
use std::convert::TryInto;

pub type GQLSchema = RootNode<'static, Query, QueryMut, EmptySubscription<Context>>;

pub fn schema() -> GQLSchema {
    GQLSchema::new(Query, QueryMut, EmptySubscription::new())
}

pub struct QueryMut;

#[graphql_object(context = Context)]
impl QueryMut {
    async fn add_schema(context: &Context, new: NewSchema) -> FieldResult<Schema> {
        let mut conn = context.connect_to_registry().await?;

        let NewSchema {
            name,
            query_address,
            topic,
            definition,
            schema_type,
        } = new;

        let rpc_schema_type: i32 = schema_type.into();

        let id = conn
            .add_schema(rpc::schema_registry::NewSchema {
                id: "".into(),
                name: name.clone(),
                query_address: query_address.clone(),
                topic: topic.clone(),
                definition,
                schema_type: rpc_schema_type,
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .id
            .parse()?;

        Ok(Schema {
            id,
            name,
            query_address,
            topic,
            schema_type,
            definitions: vec![],
            views: vec![],
        })
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    // TODO: This code is highly inefficient.
    // TODO: Views
    async fn schemas(context: &Context) -> FieldResult<Vec<Schema>> {
        let schema_names = {
            let mut conn = context.connect_to_registry().await?;
            conn.get_all_schema_names(Empty {})
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner()
                .names
        }
        .into_iter();

        let schemas: Result<_> = futures::stream::iter(schema_names)
            .then(move |(id, name): (String, String)| async move {
                let mut conn = context.connect_to_registry().await?;
                let rpc_id = rpc::schema_registry::Id { id: id.clone() };

                let topic = conn
                    .get_schema_topic(rpc_id.clone())
                    .await
                    .map_err(rpc::error::registry_error)?
                    .into_inner()
                    .topic;

                let query_address = conn
                    .get_schema_query_address(rpc_id.clone())
                    .await
                    .map_err(rpc::error::registry_error)?
                    .into_inner()
                    .address;

                let schema_type = conn
                    .get_schema_type(rpc_id.clone())
                    .await
                    .map_err(rpc::error::registry_error)?
                    .into_inner()
                    .schema_type
                    .try_into()?;

                let versions = conn
                    .get_schema_versions(rpc_id.clone())
                    .await
                    .map_err(rpc::error::registry_error)?
                    .into_inner()
                    .versions;

                let mut definitions = vec![];
                for version in versions {
                    let schema_def = conn
                        .get_schema(rpc::schema_registry::VersionedId {
                            id: id.clone(),
                            version_req: version,
                        })
                        .await
                        .map_err(rpc::error::registry_error)?
                        .into_inner();

                    definitions.push(Definition {
                        version: schema_def.version,
                        definition: schema_def.definition,
                    });
                }

                let views = vec![];
                Ok(Schema {
                    name,
                    id: id.parse()?,
                    schema_type,
                    topic,
                    query_address,

                    views,
                    definitions,
                })
            })
            .try_collect()
            .await;

        Ok(schemas?)
    }
}
