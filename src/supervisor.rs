#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SupervisorStrategy {
    #[default]
    ///stop the actor on failure (default)
    Stop,
    ///restart the actor on failure
    Restart,
    ///escalate to parent supervisor
    Escalate,
}
