use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct GenericMessage {
    pub object_id: Uuid,
    pub schema_id: Uuid,
    pub timestamp: i64,
    pub payload: Vec<u8>,
}
