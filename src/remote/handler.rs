use std::{collections::HashMap, sync::Arc};

use bytes::BytesMut;
use prost::Message as ProstMessage;

use crate::{remote::proto::Envelope, Actor, Addr, Handler};

use super::{EnvelopeHandler, RemoteMessage};

/// Create request-response handler for actor/message pair
/// Both the message M and its result M::Result must be RemoteMessage (protobuf)
pub fn make_handler<A, M>(addr: Addr<A>, node_id: &str) -> EnvelopeHandler
where
    A: Actor + Handler<M>,
    M: RemoteMessage,
    M::Result: RemoteMessage,
{
    let node_id = node_id.to_string();
    Arc::new(move |envelope: Envelope| {
        let addr = addr.clone();
        let node_id = node_id.clone();
        Box::pin(async move {
            // 1. Decode incoming message
            let msg = M::decode(envelope.payload.as_slice()).ok()?;

            // 2. Send to actor, get result
            let result = addr.send(msg).await.ok()?;

            // 3. Encode result as protobuf
            let mut buf = BytesMut::new();
            result.encode(&mut buf).ok()?;

            // 4. Build response envelope
            Some(Envelope {
                message_type: <M::Result as RemoteMessage>::type_id().to_string(),
                payload: buf.to_vec(),
                correlation_id: envelope.correlation_id,
                sender_node: node_id,
                target_actor: envelope.sender_node.clone(),
                is_response: true,
            })
        })
    })
}

/// Fire-and-forget handler (no response sent back)
pub fn make_tell_handler<A, M>(addr: Addr<A>) -> EnvelopeHandler
where
    A: Actor + Handler<M>,
    M: RemoteMessage,
{
    Arc::new(move |envelope: Envelope| {
        let addr = addr.clone();
        Box::pin(async move {
            if let Ok(msg) = M::decode(envelope.payload.as_slice()) {
                let _ = addr.do_send(msg);
            }
            None // no response
        })
    })
}

/// Router dispatches envelopes to handlers based on message_type
pub struct MessageRouter {
    handlers: HashMap<String, EnvelopeHandler>,
    default_handler: Option<EnvelopeHandler>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            default_handler: None,
        }
    }

    /// Route messages of type M to this handler
    pub fn route<M: RemoteMessage>(mut self, handler: EnvelopeHandler) -> Self {
        self.handlers.insert(M::type_id().to_string(), handler);
        self
    }

    /// Route by explicit type string
    pub fn route_type(mut self, type_id: &str, handler: EnvelopeHandler) -> Self {
        self.handlers.insert(type_id.to_string(), handler);
        self
    }

    /// Fallback for unknown message types
    pub fn default(mut self, handler: EnvelopeHandler) -> Self {
        self.default_handler = Some(handler);
        self
    }

    /// Build into a single EnvelopeHandler
    pub fn build(self) -> EnvelopeHandler {
        let handlers = Arc::new(self.handlers);
        let default = self.default_handler;

        Arc::new(move |envelope: Envelope| {
            let handlers = handlers.clone();
            let default = default.clone();

            Box::pin(async move {
                if let Some(handler) = handlers.get(&envelope.message_type) {
                    handler(envelope).await
                } else if let Some(ref default_handler) = default {
                    default_handler(envelope).await
                } else {
                    eprintln!("No handler for message type: {}", envelope.message_type);
                    None
                }
            })
        })
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}
