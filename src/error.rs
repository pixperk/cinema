#[derive(Debug, PartialEq, Eq)]
pub enum MailboxError {
    ///Actor's mailbox is closed (actor has stopped)
    MailboxClosed,
    ///Requested operation timed out
    Timeout,
    ///Actor's mailbox is full (bounded channel at capacity)
    MailboxFull,
}

impl std::fmt::Display for MailboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailboxError::MailboxClosed => write!(f, "Actor's mailbox is closed"),
            MailboxError::Timeout => write!(f, "Requested operation timed out"),
            MailboxError::MailboxFull => write!(f, "Actor's mailbox is full"),
        }
    }
}

impl std::error::Error for MailboxError {}
