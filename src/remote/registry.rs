use std::{any::Any, collections::HashMap, sync::RwLock};

use crate::remote::RemoteMessage;

type DeserializeFn = fn(&[u8]) -> Result<Box<dyn Any + Send>, prost::DecodeError>;

///Global registry for remote message types
static REGISTRY: RwLock<Option<HashMap<String, DeserializeFn>>> = RwLock::new(None);

///register a remote message type for deserialization
pub fn register_message<M: RemoteMessage + 'static>() {
    let mut registry = match REGISTRY.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let map = registry.get_or_insert_with(HashMap::new);

    map.insert(M::type_id().to_string(), |bytes| {
        let msg = M::decode(bytes)?;
        Ok(Box::new(msg))
    });
}

///deserialize a payload into a remote message
pub fn deserialize_payload(
    type_id: &str,
    payload: &[u8],
) -> Result<Box<dyn Any + Send>, prost::DecodeError> {
    let registry = match REGISTRY.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let map = registry
        .as_ref()
        .ok_or_else(|| prost::DecodeError::new("No messages registered"))?;

    let deserialize_fn = map
        .get(type_id)
        .ok_or_else(|| prost::DecodeError::new("Unknown message type"))?;

    deserialize_fn(payload)
}
