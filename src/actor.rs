use crate::{Context, Message};

//it is an entity which has own state, also
//it's size is to be known during compile time
pub trait Actor: Send + Sized + 'static {
    fn started(&mut self, ctx: &mut Context<Self>) {}
    fn stopped(&mut self, ctx: &mut Context<Self>) {}
}

/// Defines how an actor handles a specific message type.
/// One actor can handle multiple message types
pub trait Handler<M: Message>: Actor {
    fn handle(&mut self, msg: M, ctx: &mut Context<Self>) -> M::Result;
}
