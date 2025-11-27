//! Collaborative Editing WebSocket Server
//! Phase 7: Real-time multi-user timeline collaboration

use collaboration::*;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

type Tx = mpsc::UnboundedSender<Message>;
type SessionMap = Arc<RwLock<HashMap<SessionId, Session>>>;

/// Server session state
struct Session {
    _id: SessionId,
    users: HashMap<UserId, UserConnection>,
    crdt: CRDTTimeline,
}

/// User connection
struct UserConnection {
    _user: User,
    tx: Tx,
}

impl Session {
    fn new(session_id: SessionId, creator_id: UserId) -> Self {
        Self {
            _id: session_id,
            users: HashMap::new(),
            crdt: CRDTTimeline::new(session_id, creator_id),
        }
    }

    fn add_user(&mut self, user: User, tx: Tx) {
        let user_id = user.id;
        self.users
            .insert(user_id, UserConnection { _user: user, tx });
    }

    fn remove_user(&mut self, user_id: &UserId) {
        self.users.remove(user_id);
    }

    async fn broadcast(&self, msg: &SyncMessage, exclude_user: Option<UserId>) {
        let json = match serde_json::to_string(msg) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return;
            }
        };

        for (user_id, conn) in &self.users {
            if Some(*user_id) == exclude_user {
                continue;
            }

            if let Err(e) = conn.tx.send(Message::Text(json.clone())) {
                error!("Failed to send to user {}: {}", user_id.0, e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("collab_server=debug,collaboration=debug")
        .init();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    info!("Collaboration server listening on: {}", addr);

    let sessions: SessionMap = Arc::new(RwLock::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);
        tokio::spawn(handle_connection(stream, addr, sessions.clone()));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, sessions: SessionMap) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    info!("WebSocket connection established: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Task to send messages to client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(msg).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Handle incoming messages
    let mut session_id: Option<SessionId> = None;
    let mut user_id: Option<UserId> = None;

    while let Some(msg) = ws_receiver.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                error!("Error receiving message: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let result =
                    handle_sync_message(&text, &sessions, &tx, &mut session_id, &mut user_id).await;

                if let Err(e) = result {
                    error!("Error handling message: {}", e);
                    let error_msg = SyncMessage::Error {
                        message: e.to_string(),
                    };
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = tx.send(Message::Text(json));
                    }
                }
            }
            Message::Ping(data) => {
                let _ = tx.send(Message::Pong(data));
            }
            Message::Close(_) => {
                info!("Client requested close");
                break;
            }
            _ => {}
        }
    }

    // Cleanup on disconnect
    if let (Some(sid), Some(uid)) = (session_id, user_id) {
        let mut sessions_lock = sessions.write().await;
        if let Some(session) = sessions_lock.get_mut(&sid) {
            info!("User {} left session {}", uid.0, sid.0);
            session.remove_user(&uid);

            // Broadcast user left
            let msg = SyncMessage::Presence {
                update: PresenceUpdate::UserLeft { user_id: uid },
            };
            session.broadcast(&msg, None).await;

            // Remove session if empty
            if session.users.is_empty() {
                info!("Session {} is empty, removing", sid.0);
                sessions_lock.remove(&sid);
            }
        }
    }

    send_task.abort();
    info!("Connection closed: {}", addr);
}

async fn handle_sync_message(
    text: &str,
    sessions: &SessionMap,
    tx: &Tx,
    current_session: &mut Option<SessionId>,
    current_user: &mut Option<UserId>,
) -> anyhow::Result<()> {
    let msg: SyncMessage = serde_json::from_str(text)?;

    match msg {
        SyncMessage::Connect {
            session_id,
            user,
            vector_clock: _,
        } => {
            info!("User {} connecting to session {}", user.name, session_id.0);

            let mut sessions_lock = sessions.write().await;

            // Create session if it doesn't exist
            if !sessions_lock.contains_key(&session_id) {
                info!("Creating new session: {}", session_id.0);
                sessions_lock.insert(session_id, Session::new(session_id, user.id));
            }

            let session = sessions_lock.get_mut(&session_id).unwrap();

            // Get initial state (all operations)
            let initial_ops = session.crdt.get_operations_since(&VectorClock::new());

            // Add user to session
            session.add_user(user.clone(), tx.clone());
            *current_session = Some(session_id);
            *current_user = Some(user.id);

            // Send connected confirmation
            let connected_msg = SyncMessage::Connected {
                session_id,
                user_id: user.id,
                initial_state: initial_ops,
            };
            let json = serde_json::to_string(&connected_msg)?;
            tx.send(Message::Text(json))?;

            // Broadcast user joined to others
            let presence_msg = SyncMessage::Presence {
                update: PresenceUpdate::UserJoined { user: user.clone() },
            };
            session.broadcast(&presence_msg, Some(user.id)).await;

            info!("User {} joined session {}", user.name, session_id.0);
        }

        SyncMessage::Operation { operation } => {
            let session_id = current_session
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Not connected to any session"))?;

            let mut sessions_lock = sessions.write().await;
            let session = sessions_lock
                .get_mut(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

            // Apply operation to CRDT
            session
                .crdt
                .apply_remote_operation(operation.clone())
                .map_err(|e| anyhow::anyhow!("Failed to apply operation: {}", e))?;

            // Broadcast to all other users
            let broadcast_msg = SyncMessage::Operation {
                operation: operation.clone(),
            };
            session.broadcast(&broadcast_msg, *current_user).await;

            // Send acknowledgment
            let ack_msg = SyncMessage::OperationAck {
                operation_id: operation.id,
            };
            let json = serde_json::to_string(&ack_msg)?;
            tx.send(Message::Text(json))?;

            info!("Operation {} applied and broadcast", operation.id.0);
        }

        SyncMessage::SyncRequest { since } => {
            let session_id = current_session
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Not connected to any session"))?;

            let sessions_lock = sessions.read().await;
            let session = sessions_lock
                .get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

            let operations = session.crdt.get_operations_since(&since);
            let vector_clock = session.crdt.vector_clock.clone();

            let response = SyncMessage::SyncResponse {
                operations,
                vector_clock,
            };
            let json = serde_json::to_string(&response)?;
            tx.send(Message::Text(json))?;
        }

        SyncMessage::Presence { update } => {
            let session_id = current_session
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Not connected to any session"))?;

            let sessions_lock = sessions.read().await;
            let session = sessions_lock
                .get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

            // Broadcast presence update to all users
            let broadcast_msg = SyncMessage::Presence { update };
            session.broadcast(&broadcast_msg, *current_user).await;
        }

        SyncMessage::Ping => {
            let pong = SyncMessage::Pong;
            let json = serde_json::to_string(&pong)?;
            tx.send(Message::Text(json))?;
        }

        SyncMessage::Pong => {
            // Heartbeat response received
        }

        _ => {
            warn!("Unhandled message type");
        }
    }

    Ok(())
}
