/// WebSocket-based synchronization protocol for real-time collaboration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

use crate::{
    CollaborationError, PresenceUpdate, Result, SessionId, TimelineOperation, UserId,
    VectorClock, User,
};

/// Message types exchanged between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    // Connection management
    #[serde(rename = "connect")]
    Connect {
        session_id: SessionId,
        user: User,
        vector_clock: VectorClock,
    },

    #[serde(rename = "connected")]
    Connected {
        session_id: SessionId,
        user_id: UserId,
        initial_state: Vec<TimelineOperation>,
    },

    #[serde(rename = "disconnect")]
    Disconnect { user_id: UserId },

    // Operation synchronization
    #[serde(rename = "operation")]
    Operation { operation: TimelineOperation },

    #[serde(rename = "operation_ack")]
    OperationAck { operation_id: crate::OperationId },

    #[serde(rename = "sync_request")]
    SyncRequest { since: VectorClock },

    #[serde(rename = "sync_response")]
    SyncResponse {
        operations: Vec<TimelineOperation>,
        vector_clock: VectorClock,
    },

    // Presence updates
    #[serde(rename = "presence")]
    Presence { update: PresenceUpdate },

    // Error handling
    #[serde(rename = "error")]
    Error { message: String },

    // Heartbeat
    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,
}

/// Client-side sync manager
pub struct SyncClient {
    session_id: SessionId,
    user_id: UserId,
    vector_clock: VectorClock,
    pending_operations: Vec<TimelineOperation>,
    tx: mpsc::UnboundedSender<SyncMessage>,
    rx: mpsc::UnboundedReceiver<SyncMessage>,
}

impl SyncClient {
    pub fn new(session_id: SessionId, user_id: UserId) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            session_id,
            user_id,
            vector_clock: VectorClock::new(),
            pending_operations: Vec::new(),
            tx,
            rx,
        }
    }

    /// Send an operation to the server
    pub fn send_operation(&mut self, operation: TimelineOperation) -> Result<()> {
        self.pending_operations.push(operation.clone());

        self.tx
            .send(SyncMessage::Operation { operation })
            .map_err(|e| CollaborationError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Send a presence update
    pub fn send_presence(&self, update: PresenceUpdate) -> Result<()> {
        self.tx
            .send(SyncMessage::Presence { update })
            .map_err(|e| CollaborationError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Request sync from server
    pub fn request_sync(&self) -> Result<()> {
        self.tx
            .send(SyncMessage::SyncRequest {
                since: self.vector_clock.clone(),
            })
            .map_err(|e| CollaborationError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Receive next message from server
    pub async fn receive_message(&mut self) -> Option<SyncMessage> {
        self.rx.recv().await
    }

    /// Update vector clock
    pub fn update_vector_clock(&mut self, clock: VectorClock) {
        self.vector_clock.merge(&clock);
    }

    /// Get pending operations
    pub fn get_pending_operations(&self) -> &[TimelineOperation] {
        &self.pending_operations
    }

    /// Clear pending operations (after they're acknowledged)
    pub fn clear_pending_operations(&mut self) {
        self.pending_operations.clear();
    }
}

/// Server-side session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, session_id: SessionId) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if sessions.contains_key(&session_id) {
            return Err(CollaborationError::InvalidOp(
                "Session already exists".to_string(),
            ));
        }

        sessions.insert(session_id, Session::new(session_id));
        Ok(())
    }

    /// Join a session
    pub async fn join_session(
        &self,
        session_id: SessionId,
        user: User,
    ) -> Result<mpsc::UnboundedReceiver<SyncMessage>> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| CollaborationError::SessionNotFound(session_id.0.to_string()))?;

        session.add_user(user)
    }

    /// Leave a session
    pub async fn leave_session(&self, session_id: SessionId, user_id: UserId) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| CollaborationError::SessionNotFound(session_id.0.to_string()))?;

        session.remove_user(user_id);
        Ok(())
    }

    /// Broadcast operation to all users in session
    pub async fn broadcast_operation(
        &self,
        session_id: SessionId,
        operation: TimelineOperation,
        exclude_user: Option<UserId>,
    ) -> Result<()> {
        let sessions = self.sessions.read().await;

        let session = sessions
            .get(&session_id)
            .ok_or_else(|| CollaborationError::SessionNotFound(session_id.0.to_string()))?;

        session.broadcast_operation(operation, exclude_user)
    }

    /// Broadcast presence update
    pub async fn broadcast_presence(
        &self,
        session_id: SessionId,
        update: PresenceUpdate,
        exclude_user: Option<UserId>,
    ) -> Result<()> {
        let sessions = self.sessions.read().await;

        let session = sessions
            .get(&session_id)
            .ok_or_else(|| CollaborationError::SessionNotFound(session_id.0.to_string()))?;

        session.broadcast_presence(update, exclude_user)
    }

    /// Get session info
    pub async fn get_session(&self, session_id: SessionId) -> Option<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).map(|s| s.get_info())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual collaborative session
struct Session {
    id: SessionId,
    users: HashMap<UserId, mpsc::UnboundedSender<SyncMessage>>,
    operation_log: crate::OperationLog,
    vector_clock: VectorClock,
}

impl Session {
    fn new(id: SessionId) -> Self {
        Self {
            id,
            users: HashMap::new(),
            operation_log: crate::OperationLog::new(),
            vector_clock: VectorClock::new(),
        }
    }

    fn add_user(&mut self, user: User) -> Result<mpsc::UnboundedReceiver<SyncMessage>> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Send initial state to new user
        let initial_operations = self.operation_log.operations.clone();
        tx.send(SyncMessage::Connected {
            session_id: self.id,
            user_id: user.id,
            initial_state: initial_operations,
        })
        .map_err(|e| CollaborationError::NetworkError(e.to_string()))?;

        // Notify other users
        let join_msg = SyncMessage::Presence {
            update: PresenceUpdate::UserJoined { user: user.clone() },
        };

        for other_tx in self.users.values() {
            let _ = other_tx.send(join_msg.clone());
        }

        self.users.insert(user.id, tx);

        Ok(rx)
    }

    fn remove_user(&mut self, user_id: UserId) {
        self.users.remove(&user_id);

        // Notify remaining users
        let leave_msg = SyncMessage::Presence {
            update: PresenceUpdate::UserLeft { user_id },
        };

        for tx in self.users.values() {
            let _ = tx.send(leave_msg.clone());
        }
    }

    fn broadcast_operation(
        &self,
        operation: TimelineOperation,
        exclude_user: Option<UserId>,
    ) -> Result<()> {
        let msg = SyncMessage::Operation { operation };

        for (user_id, tx) in &self.users {
            if exclude_user == Some(*user_id) {
                continue;
            }

            tx.send(msg.clone())
                .map_err(|e| CollaborationError::NetworkError(e.to_string()))?;
        }

        Ok(())
    }

    fn broadcast_presence(
        &self,
        update: PresenceUpdate,
        exclude_user: Option<UserId>,
    ) -> Result<()> {
        let msg = SyncMessage::Presence { update };

        for (user_id, tx) in &self.users {
            if exclude_user == Some(*user_id) {
                continue;
            }

            tx.send(msg.clone())
                .map_err(|e| CollaborationError::NetworkError(e.to_string()))?;
        }

        Ok(())
    }

    fn get_info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id,
            user_count: self.users.len(),
            operation_count: self.operation_log.operations.len(),
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionId,
    pub user_count: usize,
    pub operation_count: usize,
}

/// WebSocket server for collaboration
pub struct CollaborationServer {
    session_manager: SessionManager,
}

impl CollaborationServer {
    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new(),
        }
    }

    /// Handle incoming WebSocket message
    pub async fn handle_message(
        &self,
        session_id: SessionId,
        user_id: UserId,
        message: SyncMessage,
    ) -> Result<()> {
        match message {
            SyncMessage::Operation { operation } => {
                // Broadcast to other users
                self.session_manager
                    .broadcast_operation(session_id, operation, Some(user_id))
                    .await?;
            }

            SyncMessage::Presence { update } => {
                // Broadcast presence update
                self.session_manager
                    .broadcast_presence(session_id, update, Some(user_id))
                    .await?;
            }

            SyncMessage::SyncRequest { since: _ } => {
                // Handle sync request
                // TODO: Send operations since the given vector clock
            }

            SyncMessage::Ping => {
                // Respond with pong
                // TODO: Send pong back to client
            }

            _ => {}
        }

        Ok(())
    }

    /// Create a new session
    pub async fn create_session(&self, session_id: SessionId) -> Result<()> {
        self.session_manager.create_session(session_id).await
    }

    /// User joins a session
    pub async fn join_session(
        &self,
        session_id: SessionId,
        user: User,
    ) -> Result<mpsc::UnboundedReceiver<SyncMessage>> {
        self.session_manager.join_session(session_id, user).await
    }

    /// User leaves a session
    pub async fn leave_session(&self, session_id: SessionId, user_id: UserId) -> Result<()> {
        self.session_manager.leave_session(session_id, user_id).await
    }
}

impl Default for CollaborationServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LamportClock, OperationKind};
    use timeline::{ClipNode, Frame, FrameRange, NodeId, TimelineNode, TimelineNodeKind};

    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new();
        let session_id = SessionId::new();

        let result = manager.create_session(session_id).await;
        assert!(result.is_ok());

        // Try creating again - should fail
        let result = manager.create_session(session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_join_session() {
        let manager = SessionManager::new();
        let session_id = SessionId::new();

        manager.create_session(session_id).await.unwrap();

        let user = User {
            id: UserId::new(),
            name: "Alice".to_string(),
            color: crate::UserColor::new(255, 0, 0),
            avatar_url: None,
        };

        let result = manager.join_session(session_id, user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_operation() {
        let manager = SessionManager::new();
        let session_id = SessionId::new();

        manager.create_session(session_id).await.unwrap();

        let user1 = User {
            id: UserId::new(),
            name: "Alice".to_string(),
            color: crate::UserColor::new(255, 0, 0),
            avatar_url: None,
        };

        let user2 = User {
            id: UserId::new(),
            name: "Bob".to_string(),
            color: crate::UserColor::new(0, 255, 0),
            avatar_url: None,
        };

        let mut _rx1 = manager.join_session(session_id, user1.clone()).await.unwrap();
        let mut rx2 = manager.join_session(session_id, user2.clone()).await.unwrap();

        // Skip the initial Connected message
        let msg = rx2.recv().await;
        assert!(matches!(msg, Some(SyncMessage::Connected { .. })));

        // Create an operation
        let node = TimelineNode {
            id: NodeId::new(),
            label: Some("Test Clip".to_string()),
            kind: TimelineNodeKind::Clip(ClipNode {
                asset_id: Some("test.mp4".to_string()),
                media_range: FrameRange {
                    start: 0,
                    duration: 300,
                },
                timeline_range: FrameRange {
                    start: 0,
                    duration: 100,
                },
                playback_rate: 1.0,
                reverse: false,
                metadata: serde_json::Value::Null,
            }),
            locked: false,
            metadata: serde_json::Value::Null,
        };

        let operation = TimelineOperation::new(
            user1.id,
            LamportClock(1),
            OperationKind::AddNode { node },
        );

        // Broadcast operation
        manager
            .broadcast_operation(session_id, operation, Some(user1.id))
            .await
            .unwrap();

        // User2 should receive the operation
        tokio::select! {
            msg = rx2.recv() => {
                assert!(matches!(msg, Some(SyncMessage::Operation { .. })), "Expected Operation message, got {:?}", msg);
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                panic!("Timeout waiting for operation");
            }
        }
    }
}
