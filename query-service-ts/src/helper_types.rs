use crate::schema::DataPoint;
use serde::Serialize;

#[derive(Serialize)]
#[serde(remote = "DataPoint")]
pub struct DataPointDef {
    timestamp: String,
    value: f32,
}

#[derive(Serialize)]
pub struct DataPointSerializable(#[serde(with = "DataPointDef")] pub DataPoint);