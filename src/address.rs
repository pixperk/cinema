use tokio::sync::{mpsc, oneshot};

use crate::{
    actor::ActorId,
    envelope::{Envelope, MessageEnvelope},
    error::MailboxError,
    message::Terminated,
    watcher::Watcher,
    Actor, Handler, Message,
};

pub struct Addr<A: Actor> {
    sender: mpsc::UnboundedSender<Box<dyn Envelope<A>>>,
    id: ActorId,
}

impl<A: Actor> Addr<A> {
    pub fn new(sender: mpsc::UnboundedSender<Box<dyn Envelope<A>>>, id: ActorId) -> Self {
        Self { sender, id }
    }

    pub fn id(&self) -> ActorId {
        self.id
    }

    ///Send message and wait for response
    pub async fn send<M>(&self, msg: M) -> Result<M::Result, MailboxError>
    where
        A: Handler<M>,
        M: Message,
    {
        let (tx, rx) = oneshot::channel();
        let envelope = MessageEnvelope::with_response(msg, tx);
        self.sender
            .send(Box::new(envelope))
            .map_err(|_| MailboxError::MailboxClosed)?;

        rx.await.map_err(|_| MailboxError::MailboxClosed)
    }

    pub async fn send_timeout<M>(
        &self,
        msg: M,
        timeout: std::time::Duration,
    ) -> Result<M::Result, MailboxError>
    where
        A: Handler<M>,
        M: Message,
    {
        let (tx, rx) = oneshot::channel();
        let envelope = MessageEnvelope::with_response(msg, tx);
        self.sender
            .send(Box::new(envelope))
            .map_err(|_| MailboxError::MailboxClosed)?;

        match tokio::time::timeout(timeout, rx).await {
            Ok(res) => res.map_err(|_| MailboxError::MailboxClosed),
            Err(_) => Err(MailboxError::Timeout),
        }
    }

    ///Fire and forget message sending
    pub fn do_send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message,
    {
        let envelope = MessageEnvelope::new(msg);
        let _ = self.sender.send(Box::new(envelope));
    }

    ///Check if the actor is still alive
    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            id: self.id,
        }
    }
}

impl<A> Watcher for Addr<A>
where
    A: Actor + Handler<Terminated>,
{
    fn notify(&self, id: ActorId) {
        self.do_send(Terminated { id });
    }
}
