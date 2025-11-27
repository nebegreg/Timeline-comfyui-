/// Phase 7: Collaborative Editing Infrastructure
/// Real-time multi-user timeline collaboration using CRDT
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod operations;
pub use operations::*;

mod sync;
pub use sync::*;

mod presence;
pub use presence::*;

mod crdt;
pub use crdt::*;

mod conflict;
pub use conflict::*;

mod offline;
pub use offline::*;

#[derive(Debug, Error)]
pub enum CollaborationError {
    #[error("sync error: {0}")]
    SyncError(String),

    #[error("operation conflict: {0}")]
    ConflictError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("invalid operation: {0}")]
    InvalidOp(String),
}

pub type Result<T> = std::result::Result<T, CollaborationError>;

/// User identifier in collaborative session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub uuid::Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

/// Session identifier for collaborative editing session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub uuid::Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Lamport timestamp for causality tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportClock(pub u64);

impl LamportClock {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn tick(&mut self) {
        self.0 += 1;
    }

    pub fn update(&mut self, other: LamportClock) {
        self.0 = self.0.max(other.0) + 1;
    }
}

impl Default for LamportClock {
    fn default() -> Self {
        Self::new()
    }
}
