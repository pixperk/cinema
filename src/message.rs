///A message is something that can be sent to an actor
pub trait Message: Send + 'static {
    type Result: Send;
}
