use std::marker::PhantomData;

///unique identifier for a remote node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

///address of a remote actor
#[derive(Debug, Clone)]
pub struct RemoteActorId {
    pub node: NodeId,
    pub actor_name: String,
}

///remote address - points to an actor on another node
pub struct RemoteAddr<A> {
    pub id: RemoteActorId,
    pub node_addr: String,
    _phantom: PhantomData<A>,
}

impl<A> RemoteAddr<A> {
    pub fn new(node_id: &str, actor_name: &str, node_addr: &str) -> Self {
        Self {
            id: RemoteActorId {
                node: NodeId(node_id.to_string()),
                actor_name: actor_name.to_string(),
            },
            node_addr: node_addr.to_string(),
            _phantom: PhantomData,
        }
    }
}
