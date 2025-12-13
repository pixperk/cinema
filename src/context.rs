use std::{ops::Add, sync::Arc};

use tokio::sync::Notify;

use crate::{actor::ActorId, message::Terminated, watcher::Watcher, Actor, Addr, Handler};

///Runtime context for an actor
pub struct Context<A: Actor> {
    addr: Addr<A>,
    ///signal to stop the actor
    stop_signal: Option<Arc<Notify>>,
}

impl<A: Actor> Context<A> {
    pub fn new(addr: Addr<A>) -> Self {
        Self {
            addr,
            stop_signal: None,
        }
    }

    ///configure the context with a stop signal for graceful shutdown
    pub fn with_stop_signal(addr: Addr<A>, stop_signal: Arc<Notify>) -> Self {
        Self {
            addr,
            stop_signal: Some(stop_signal),
        }
    }

    ///Get the address of the actor associated with this context
    pub fn address(&self) -> Addr<A> {
        self.addr.clone()
    }

    /// Get this actor's ID
    pub fn id(&self) -> ActorId {
        self.addr.id()
    }

    ///stop the actor associated with this context
    pub fn stop(&self) {
        if let Some(signal) = &self.stop_signal {
            signal.notify_one();
        }
    }

    /// Watch another actor - receive Terminated when it dies
    /// When the watched actor stops, this actor will receive
    /// a Terminated message with the dead actor's ID
    pub fn watch<B>(&self, addr: &Addr<B>)
    where
        B: Actor,
        A: Handler<Terminated>,
    {
        addr.watch(self.addr.clone());
    }
}
