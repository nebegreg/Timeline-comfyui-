use collaboration::{
    PresenceUpdate, SessionId, SyncMessage, TimelineOperation, User,
    UserId, UserPresence, VectorClock,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

/// A collaboration session
pub struct Session {
    pub id: SessionId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub users: DashMap<UserId, UserInfo>,
    pub operation_log: Arc<tokio::sync::RwLock<Vec<TimelineOperation>>>,
    pub broadcast_tx: broadcast::Sender<SyncMessage>,
}

#[derive(Clone)]
pub struct UserInfo {
    pub user: User,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub presence: UserPresence,
}

impl Session {
    pub fn new(id: SessionId) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        Self {
            id,
            created_at: chrono::Utc::now(),
            users: DashMap::new(),
            operation_log: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            broadcast_tx,
        }
    }

    pub async fn add_user(&self, user_id: UserId, user: User) {
        let user_info = UserInfo {
            user: user.clone(),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            presence: UserPresence::new(user.clone(), self.id),
        };
        self.users.insert(user_id, user_info);

        // Broadcast user joined
        let _ = self.broadcast_tx.send(SyncMessage::Presence {
            update: PresenceUpdate::UserJoined { user },
        });
    }

    pub async fn remove_user(&self, user_id: UserId) {
        if let Some((_, _user_info)) = self.users.remove(&user_id) {
            let _ = self.broadcast_tx.send(SyncMessage::Presence {
                update: PresenceUpdate::UserLeft { user_id },
            });
        }
    }

    pub async fn add_operation(&self, operation: TimelineOperation) {
        let mut log = self.operation_log.write().await;
        log.push(operation.clone());

        // Broadcast to all users
        let _ = self.broadcast_tx.send(SyncMessage::Operation { operation });
    }

    pub async fn get_operations_since(&self, vector_clock: &VectorClock) -> Vec<TimelineOperation> {
        let log = self.operation_log.read().await;

        // Filter operations that are newer than the provided vector clock
        log.iter()
            .filter(|op| {
                // If the operation's user has a higher clock value, include it
                let clock_value = vector_clock.get(op.user_id);
                op.clock.0 > clock_value
            })
            .cloned()
            .collect()
    }

    pub async fn get_initial_state(&self) -> Vec<TimelineOperation> {
        let log = self.operation_log.read().await;
        log.clone()
    }

    pub async fn update_presence(&self, user_id: UserId, update: PresenceUpdate) {
        if let Some(mut user_info) = self.users.get_mut(&user_id) {
            user_info.last_activity = chrono::Utc::now();

            // Update presence based on the update type
            match &update {
                PresenceUpdate::CursorMoved { position, .. } => {
                    user_info.presence.cursor_position = Some(position.clone());
                }
                PresenceUpdate::SelectionChanged { selection, .. } => {
                    user_info.presence.selection = Some(selection.clone());
                }
                PresenceUpdate::ViewportChanged { viewport, .. } => {
                    user_info.presence.viewport = Some(viewport.clone());
                }
                _ => {}
            }
        }

        // Broadcast presence update
        let _ = self.broadcast_tx.send(SyncMessage::Presence { update });
    }

    pub fn get_active_users(&self) -> Vec<User> {
        self.users
            .iter()
            .map(|entry| entry.value().user.clone())
            .collect()
    }
}

/// Manages all collaboration sessions
pub struct SessionManager {
    sessions: DashMap<SessionId, Arc<Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub async fn create_session(&self, session_id: SessionId) -> Arc<Session> {
        let session = Arc::new(Session::new(session_id));
        self.sessions.insert(session_id, session.clone());
        info!("Created session: {}", session_id.0);
        session
    }

    pub async fn get_or_create_session(&self, session_id: SessionId) -> Arc<Session> {
        if let Some(session) = self.sessions.get(&session_id) {
            session.clone()
        } else {
            self.create_session(session_id).await
        }
    }

    pub async fn get_session(&self, session_id: SessionId) -> Option<Arc<Session>> {
        self.sessions.get(&session_id).map(|s| s.clone())
    }

    pub async fn remove_session(&self, session_id: SessionId) {
        if self.sessions.remove(&session_id).is_some() {
            info!("Removed session: {}", session_id.0);
        }
    }

    pub async fn list_sessions(&self) -> Vec<serde_json::Value> {
        self.sessions
            .iter()
            .map(|entry| {
                let session = entry.value();
                let user_count = session.users.len();
                serde_json::json!({
                    "id": session.id.0,
                    "created_at": session.created_at,
                    "user_count": user_count,
                    "users": session.get_active_users(),
                })
            })
            .collect()
    }

    pub async fn get_session_info(&self, session_id: SessionId) -> Option<serde_json::Value> {
        self.sessions.get(&session_id).map(|session| {
            let users: Vec<_> = session.users.iter().map(|entry| {
                let user_info = entry.value();
                serde_json::json!({
                    "user": user_info.user,
                    "connected_at": user_info.connected_at,
                    "last_activity": user_info.last_activity,
                    "presence": user_info.presence,
                })
            }).collect();

            serde_json::json!({
                "id": session.id.0,
                "created_at": session.created_at,
                "user_count": session.users.len(),
                "users": users,
            })
        })
    }
}
