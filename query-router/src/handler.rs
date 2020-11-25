use crate::{cache::AddressCache, error::Error};
use serde_json::{Map, Value};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use warp::Buf;

pub async fn query_single(
    object_id: Uuid,
    schema_id: Uuid,
    cache: Arc<AddressCache>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let address = cache.get_address(schema_id).await?;
    let mut values = query_service::query_multiple(vec![object_id.to_string()], address)
        .await
        .map_err(Error::QueryError)?;

    Ok(warp::reply::with_header(
        values
            .remove(&object_id.to_string())
            .ok_or(Error::SingleQueryMissingValue)?,
        "Content-Type",
        "application/json",
    ))
}

pub async fn query_multiple(
    object_ids: String,
    schema_id: Uuid,
    cache: Arc<AddressCache>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let address = cache.get_address(schema_id).await?;
    let object_ids = object_ids.split(',').map(str::to_owned).collect();
    let values = query_service::query_multiple(object_ids, address)
        .await
        .map_err(Error::QueryError)?;

    Ok(warp::reply::json(&byte_map_to_json_map(values)?))
}

pub async fn query_by_schema(
    schema_id: Uuid,
    cache: Arc<AddressCache>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let address = cache.get_address(schema_id).await?;
    let values = query_service::query_by_schema(schema_id.to_string(), address)
        .await
        .map_err(Error::QueryError)?;

    Ok(warp::reply::json(&byte_map_to_json_map(values)?))
}

pub async fn query_raw(
    body: impl Buf,
    schema_id: Uuid,
    cache: Arc<AddressCache>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let msg = String::from_utf8_lossy(body.bytes()).into_owned();
    let address = cache.get_address(schema_id).await?;
    let values = query_service::query_raw(msg, address)
        .await
        .map_err(Error::QueryError)?;

    Ok(warp::reply::json(&byte_map_to_json_map(values)?))
}

fn byte_map_to_json_map(map: HashMap<String, Vec<u8>>) -> Result<Map<String, Value>, Error> {
    map.into_iter()
        .map(|(object_id, value)| {
            Ok((
                object_id,
                serde_json::from_slice(&value).map_err(Error::JsonError)?,
            ))
        })
        .collect::<Result<Map<String, Value>, Error>>()
}
