use tokio::sync::oneshot;

use crate::{Actor, Context, Handler, Message};

///Envelope acts as a type erasure for messages sent to actors
/// it is wrapped in a Box to allow for dynamic dispatch
pub trait Envelope<A: Actor>: Send {
    fn handle(self: Box<Self>, actor: &mut A, ctx: &mut Context<A>);
}

pub struct MessageEnvelope<M>
where
    M: Message,
{
    //an optional message, once taken it becomes None
    msg: Option<M>,
    response_tx: Option<oneshot::Sender<M::Result>>,
}

impl<M: Message> MessageEnvelope<M> {
    ///fire and forget message envelope (no response expected)
    pub fn new(msg: M) -> Self {
        Self {
            msg: Some(msg),
            response_tx: None,
        }
    }

    ///with response channel
    pub fn with_response(msg: M, tx: oneshot::Sender<M::Result>) -> Self {
        Self {
            msg: Some(msg),
            response_tx: Some(tx),
        }
    }
}

impl<A, M> Envelope<A> for MessageEnvelope<M>
where
    A: Actor + Handler<M>,
    M: Message,
{
    fn handle(mut self: Box<Self>, actor: &mut A, ctx: &mut Context<A>) {
        if let Some(msg) = self.msg.take() {
            let result = actor.handle(msg, ctx);

            if let Some(tx) = self.response_tx.take() {
                //error can be ignored if receiver is dropped
                let _ = tx.send(result);
            }
        }
    }
}
