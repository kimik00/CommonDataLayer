use crate::context::Context;
use crate::error::ApiResult;
use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldResult, GraphQLEnum, GraphQLObject,
    RootNode,
};
use rpc::schema_registry::Empty;
use uuid::Uuid;

pub type Schema =
    RootNode<'static, Query<Context>, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}

#[derive(GraphQLObject)]
pub struct CdlSchemaName {
    pub name: String,
    pub id: Uuid,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn schema_names(context: &Context) -> FieldResult<Vec<CdlSchemaName>> {
        let mut conn = context.connect_to_registry().await?;
        let schema_names = conn
            .get_all_schema_names(Empty {})
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner();

        Ok(schema_names
            .names
            .into_iter()
            .map(|(name, id)| {
                Ok(CdlSchemaName {
                    name,
                    id: id.parse()?,
                })
            })
            .collect::<ApiResult<Vec<CdlSchemaName>>>()?)
    }
}
