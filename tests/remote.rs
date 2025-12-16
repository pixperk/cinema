use cinema::{
    remote::{deserialize_payload, proto::Envelope, register_message, RemoteMessage},
    Message,
};
use prost::Message as ProstMessage;

#[derive(Clone, ProstMessage)]
struct Ping {
    #[prost(string, tag = "1")]
    message: String,
}

impl Message for Ping {
    type Result = ();
}

impl RemoteMessage for Ping {
    fn type_id() -> &'static str {
        "test::Ping"
    }
}

#[test]
fn envelope_roundtrip() {
    let ping = Ping {
        message: "Hello, World!".to_string(),
    };

    let envelope = Envelope::from_message(&ping, 42, "node", "actor");

    assert_eq!(envelope.message_type, "test::Ping");
    assert_eq!(envelope.correlation_id, 42);

    let serialized = envelope.to_bytes();

    let decoded = Envelope::from_bytes(&serialized).unwrap();
    assert_eq!(decoded.message_type, "test::Ping");
    assert_eq!(decoded.correlation_id, 42);
}

#[test]
fn registry_deserialize() {
    register_message::<Ping>();

    let ping = Ping {
        message: "Hello, Registry!".to_string(),
    };
    let envelope = Envelope::from_message(&ping, 1, "node", "actor");

    let deserialized = deserialize_payload(&envelope.message_type, &envelope.payload).unwrap();
    let downcasted = deserialized.downcast_ref::<Ping>().unwrap();

    assert_eq!(downcasted.message, "Hello, Registry!");
}
