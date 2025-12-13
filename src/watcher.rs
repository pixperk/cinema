use crate::actor::ActorId;

/// Type-erased watcher that can be notified of actor death
pub trait Watcher: Send + Sync {
    fn notify(&self, id: ActorId);
}
