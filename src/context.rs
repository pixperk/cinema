use std::marker::PhantomData;

use crate::Actor;

///Runtime context for an actor
pub struct Context<A: Actor> {
    _phantom: PhantomData<A>,
}

impl<A: Actor> Context<A> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
