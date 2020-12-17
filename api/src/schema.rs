use crate::context::Context;
use crate::error::Result;
use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldResult, GraphQLObject, RootNode,
};
use rpc::schema_registry::Empty;
use uuid::Uuid;

pub type GQLSchema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> GQLSchema {
    GQLSchema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}

#[derive(GraphQLObject)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn schemas(context: &Context) -> FieldResult<Vec<Schema>> {
        let mut conn = context.connect_to_registry().await?;
        let schema_names = conn
            .get_all_schema_names(Empty {})
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner();

        Ok(schema_names
            .names
            .into_iter()
            .map(|(id, name)| {
                Ok(Schema {
                    name,
                    id: id.parse()?,
                })
            })
            .collect::<Result<_>>()?)
    }
}
