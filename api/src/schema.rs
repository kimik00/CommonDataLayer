use crate::error::{Error, Result};
use std::convert::TryFrom;
use uuid::Uuid;

pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub topic: String,
    pub query_address: String,
    pub schema_type: SchemaType,
}

#[derive(Debug, juniper::GraphQLEnum, Clone, Copy)]
pub enum SchemaType {
    DocumentStorage,
    Timeseries,
}

impl TryFrom<i32> for SchemaType {
    type Error = Error;
    fn try_from(i: i32) -> Result<Self> {
        match i {
            0 => Ok(Self::DocumentStorage),
            1 => Ok(Self::Timeseries),
            i => Err(Error::InvalidSchemaType(i)),
        }
    }
}

impl Into<i32> for SchemaType {
    fn into(self) -> i32 {
        match self {
            Self::DocumentStorage => 0,
            Self::Timeseries => 1,
        }
    }
}

#[derive(juniper::GraphQLObject)]
pub struct Definition {
    pub definition: String,
    pub version: String,
    pub queried_version: String,
}

#[derive(juniper::GraphQLObject)]
pub struct View {
    pub id: Uuid,
    pub name: String,
    pub expression: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewSchema {
    pub name: String,
    pub query_address: String,
    pub topic: String,
    pub definition: String,
    #[graphql(name = "type")]
    pub schema_type: SchemaType,
}

#[derive(Clone, juniper::GraphQLInputObject)]
pub struct NewView {
    pub name: String,
    pub expression: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewVersion {
    pub version: String,
    pub definition: String,
}
