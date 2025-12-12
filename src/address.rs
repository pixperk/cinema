use std::marker::PhantomData;

use crate::Actor;

pub struct Addr<A: Actor> {
    _phantom: PhantomData<A>,
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
