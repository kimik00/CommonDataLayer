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
    async fn schemas(context: &Context) -> FieldResult<Vec<Schema>> {
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
