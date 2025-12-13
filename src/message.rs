use crate::actor::ActorId;

///A message is something that can be sent to an actor
pub trait Message: Send + 'static {
    type Result: Send;
}

/// Sent to watchers when a watched actor stops
#[derive(Debug, Clone)]
pub struct Terminated {
    pub id: ActorId,
}

impl Message for Terminated {
    type Result = ();
}
