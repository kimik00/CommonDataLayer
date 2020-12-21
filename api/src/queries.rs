use crate::context::Context;
use crate::error::Result;
use crate::schema::*;
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};
use rpc::schema_registry::Empty;
use std::convert::TryInto;
use uuid::Uuid;

pub type GQLSchema = RootNode<'static, Query, QueryMut, EmptySubscription<Context>>;

pub fn schema() -> GQLSchema {
    GQLSchema::new(Query, QueryMut, EmptySubscription::new())
}

pub struct QueryMut;

#[graphql_object(context = Context)]
impl QueryMut {
    async fn add_schema(context: &Context, new: NewSchema) -> FieldResult<Schema> {
        log::debug!("add schema {:?}", new);
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
        })
    }

    async fn add_schema_definition(
        context: &Context,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        log::debug!(
            "add schema definition for {:?} - {:?}",
            schema_id,
            new_version
        );
        let mut conn = context.connect_to_registry().await?;

        let NewVersion {
            definition,
            version,
        } = new_version;

        conn.add_schema_version(rpc::schema_registry::NewSchemaVersion {
            id: schema_id.to_string(),
            version: version.clone(),
            definition: definition.clone(),
        })
        .await
        .map_err(rpc::error::registry_error)?;

        Ok(Definition {
            queried_version: version.clone(),
            definition,
            version,
        })
    }

    async fn add_view(context: &Context, schema_id: Uuid, new_view: NewView) -> FieldResult<View> {
        log::debug!("add view for {} - {:?}", schema_id, new_view);

        let NewView { name, expression } = new_view.clone();
        let mut conn = context.connect_to_registry().await?;
        let id = conn
            .add_view_to_schema(rpc::schema_registry::NewSchemaView {
                schema_id: schema_id.to_string(),
                view_id: "".into(),
                name,
                jmespath: expression,
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .id;
        Ok(View {
            id: id.parse()?,
            name: new_view.name,
            expression: new_view.expression,
        })
    }

    async fn update_schema(
        context: &Context,
        schema_id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<Option<Schema>> {
        log::debug!("update schema for {} - {:?}", schema_id, update);

        let mut conn = context.connect_to_registry().await?;

        let UpdateSchema {
            name,
            query_address,
            topic,
            schema_type,
        } = update;

        if let Some(name) = name {
            conn.update_schema_name(rpc::schema_registry::SchemaNameUpdate {
                id: schema_id.to_string(),
                name,
            })
            .await
            .map_err(rpc::error::registry_error)?;
        }

        if let Some(address) = query_address {
            conn.update_schema_query_address(rpc::schema_registry::SchemaQueryAddressUpdate {
                id: schema_id.to_string(),
                address,
            })
            .await
            .map_err(rpc::error::registry_error)?;
        }

        if let Some(topic) = topic {
            conn.update_schema_topic(rpc::schema_registry::SchemaTopicUpdate {
                id: schema_id.to_string(),
                topic,
            })
            .await
            .map_err(rpc::error::registry_error)?;
        }

        if let Some(schema_type) = schema_type {
            let schema_type: i32 = schema_type.into();
            conn.update_schema_type(rpc::schema_registry::SchemaTypeUpdate {
                id: schema_id.to_string(),
                schema_type,
            })
            .await
            .map_err(rpc::error::registry_error)?;
        }

        drop(conn);

        get_schema(context, schema_id).await
    }
}

#[graphql_object(context = Context)]
impl Schema {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn topic(&self) -> &str {
        &self.topic
    }

    fn query_address(&self) -> &str {
        &self.query_address
    }

    #[graphql(name = "type")]
    fn schema_type(&self) -> SchemaType {
        self.schema_type
    }

    async fn definitions(&self, context: &Context) -> FieldResult<Vec<Definition>> {
        let mut conn = context.connect_to_registry().await?;
        let id = self.id.to_string();
        let rpc_id = rpc::schema_registry::Id { id: id.clone() };

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
                    version_req: version.clone(),
                })
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner();

            definitions.push(Definition {
                version: schema_def.version,
                queried_version: version,
                definition: schema_def.definition,
            });
        }

        Ok(definitions)
    }

    async fn views(&self, context: &Context) -> FieldResult<Vec<View>> {
        let mut conn = context.connect_to_registry().await?;
        let id = self.id.to_string();
        let rpc_id = rpc::schema_registry::Id { id: id.clone() };

        let views = conn
            .get_all_views_of_schema(rpc_id.clone())
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .views
            .into_iter()
            .map(|(id, view)| {
                Ok(View {
                    id: id.parse()?,
                    name: view.name,
                    expression: view.jmespath,
                })
            })
            .collect::<Result<_>>()?;

        Ok(views)
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn schema(context: &Context, id: Uuid) -> FieldResult<Option<Schema>> {
        get_schema(context, id).await
    }

    async fn schemas(context: &Context) -> FieldResult<Vec<Schema>> {
        log::debug!("get all schemas");
        let mut conn = context.connect_to_registry().await?;
        let schemas = conn
            .get_all_schemas(Empty {})
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .schemas
            .into_iter()
            .map(|(schema_id, schema)| {
                Ok(Schema {
                    id: schema_id.parse()?,
                    name: schema.name,
                    topic: schema.topic,
                    query_address: schema.query_address,
                    schema_type: schema.schema_type.try_into()?,
                })
            })
            .collect::<Result<_>>()?;

        Ok(schemas)
    }
}

async fn get_schema(context: &Context, id: Uuid) -> FieldResult<Option<Schema>> {
    log::debug!("get schema: {:?}", id);
    let mut conn = context.connect_to_registry().await?;
    let schema = conn
        .get_all_schemas(Empty {}) // TODO: Add GRPC route to schema_registry which takes schema for uuid.
        // Right now we have a route for `get_schema` but it returns SchemaDefinition, not metadata.
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner()
        .schemas
        .into_iter()
        .map(|(schema_id, schema)| {
            let schema_id: Uuid = schema_id.parse()?;
            if schema_id == id {
                Ok(Some(Schema {
                    id,
                    name: schema.name,
                    topic: schema.topic,
                    query_address: schema.query_address,
                    schema_type: schema.schema_type.try_into()?,
                }))
            } else {
                Ok(None)
            }
        })
        .find(|schema: &Result<Option<Schema>>| matches!(schema, Ok(None)))
        .transpose()?
        .flatten();

    log::debug!("schema: {:?}", schema);

    Ok(schema)
}
