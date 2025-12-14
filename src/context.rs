use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
    time::Duration,
};

use futures::FutureExt;
use tokio::sync::{mpsc, Notify};

use crate::{
    actor::ActorId, address::ChildHandle, envelope::ActorMessage, message::Terminated, Actor, Addr,
    Handler, Message, TimerHandle,
};

///Runtime context for an actor
pub struct Context<A: Actor> {
    addr: Addr<A>,
    ///signal to stop the actor
    stop_signal: Option<Arc<Notify>>,
    shutdown: Arc<Notify>,
    children: Vec<Box<dyn ChildHandle>>,
}

impl<A: Actor> Context<A> {
    pub fn new(addr: Addr<A>, shutdown: Arc<Notify>) -> Self {
        Self {
            addr,
            stop_signal: None,
            shutdown,
            children: Vec::new(),
        }
    }

    ///configure the context with a stop signal for graceful shutdown
    pub fn with_stop_signal(
        addr: Addr<A>,
        stop_signal: Arc<Notify>,
        shutdown: Arc<Notify>,
    ) -> Self {
        Self {
            addr,
            stop_signal: Some(stop_signal),
            shutdown,
            children: Vec::new(),
        }
    }

    ///Stop all child actors (when this actor stops)
    pub fn stop_children(&mut self) {
        for child in &self.children {
            child.stop();
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
        addr.add_watcher(self.addr.clone());
    }

    /// Send a message to self after delay
    /// Returns a TimerHandle that can be used to cancel the timer
    pub fn run_later<M>(&self, delay: Duration, msg: M) -> TimerHandle
    where
        M: Message,
        A: Handler<M>,
    {
        let addr = self.addr.clone();
        let handle = TimerHandle::new();
        let handle_clone = handle.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            if !handle_clone.is_cancelled() {
                addr.do_send(msg);
            }
        });

        handle
    }

    /// Send a message to self repeatedly at fixed intervals
    /// Returns a TimerHandle that can be used to cancel the interval
    pub fn run_interval<M>(&self, interval: Duration, msg: M) -> TimerHandle
    where
        M: Message + Clone,
        A: Handler<M>,
    {
        let addr = self.addr.clone();
        let handle = TimerHandle::new();
        let handle_clone = handle.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if !addr.is_alive() || handle_clone.is_cancelled() {
                    break;
                }
                addr.do_send(msg.clone());
            }
        });

        handle
    }

    ///Spawn a child actor supervised by this actor
    /// Child inherits shutdown signal from parent
    /// Stops when parent stops
    /// Parent receives Terminated message when child stops
    pub fn spawn_child<C>(&mut self, mut child: C) -> Addr<C>
    where
        C: Actor,
        A: Handler<Terminated>,
    {
        let (tx, mut rx) = mpsc::unbounded_channel::<ActorMessage<C>>();
        let child_id = ActorId::new();
        let child_stop_signal = Arc::new(Notify::new());
        let child_addr = Addr::new(tx, child_id, child_stop_signal.clone());

        let shutdown = self.shutdown.clone();
        let mut child_ctx = Context::with_stop_signal(
            child_addr.clone(),
            child_stop_signal.clone(),
            shutdown.clone(),
        );

        let child_addr_for_notify = child_addr.clone();

        tokio::spawn(async move {
            child.started(&mut child_ctx);

            let panic_occurred = loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(actor_msg) => {
                                let result = match actor_msg {
                                    ActorMessage::Sync(envelope) => {
                                        catch_unwind(AssertUnwindSafe(|| {
                                            envelope.handle(&mut child, &mut child_ctx)
                                        }))
                                    }
                                    ActorMessage::Async(envelope) => {
                                        let fut = envelope.handle(&mut child, &mut child_ctx);
                                        AssertUnwindSafe(fut).catch_unwind().await
                                    }
                                };
                                if result.is_err() {
                                    break true;
                                }
                            }
                            None => break false,
                        }
                    }
                    _ = shutdown.notified() => break false,
                    _ = child_stop_signal.notified() => break false,
                }
            };

            if panic_occurred {
                eprintln!("Child actor panicked. Stopping gracefully.");
            }

            child_addr_for_notify.notify_watchers();
            child_ctx.stop_children();
            child.stopped(&mut child_ctx);
        });

        //auto watch the child
        self.watch(&child_addr);

        //keep track of child for stopping later
        self.children.push(Box::new(child_addr.clone()));

        child_addr
    }
}
