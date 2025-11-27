use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use collaboration::{
    PresenceUpdate, SessionId, SyncMessage, TimelineOperation, User, UserId, VectorClock,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{session::Session, AppState};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    session_id: String,
    user_id: Option<String>,
    user_name: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    let session_id = query
        .session_id
        .parse::<Uuid>()
        .map(SessionId)
        .unwrap_or_else(|_| SessionId(Uuid::new_v4()));

    let user_id = query
        .user_id
        .and_then(|id| id.parse::<Uuid>().ok())
        .map(UserId)
        .unwrap_or_else(|| UserId(Uuid::new_v4()));

    let user_name = query
        .user_name
        .unwrap_or_else(|| format!("User-{}", &user_id.0.to_string()[..8]));

    ws.on_upgrade(move |socket| handle_socket(socket, state, session_id, user_id, user_name))
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    session_id: SessionId,
    user_id: UserId,
    user_name: String,
) {
    info!(
        "New WebSocket connection: session={}, user={} ({})",
        session_id.0, user_id.0, user_name
    );

    // Get or create session
    let session = state.session_manager.get_or_create_session(session_id).await;

    // Create user
    let user = User::new(user_id, user_name.clone());

    // Add user to session
    session.add_user(user_id, user.clone()).await;

    // Split socket
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Subscribe to broadcasts
    let mut broadcast_rx = session.broadcast_tx.subscribe();

    // Create a channel for sending messages to the WebSocket
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Send initial state
    let initial_ops = session.get_initial_state().await;

    let connect_msg = SyncMessage::Connected {
        session_id,
        user_id,
        initial_state: initial_ops,
    };

    if let Ok(json) = serde_json::to_string(&connect_msg) {
        if let Err(e) = ws_tx.send(Message::Text(json)).await {
            error!("Failed to send initial state: {}", e);
            return;
        }
    }

    // Spawn task to forward broadcasts to this user
    let tx_clone = tx.clone();
    let user_id_clone = user_id;
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            // Don't send back operations from this user
            let should_send = match &msg {
                SyncMessage::Operation { operation } => operation.user_id != user_id_clone,
                _ => true,
            };

            if should_send {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if tx_clone.send(Message::Text(json)).is_err() {
                        debug!("Failed to send broadcast: channel closed");
                        break;
                    }
                }
            }
        }
    });

    // Spawn task to send messages to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_tx.send(msg).await {
                debug!("Failed to send message to WebSocket: {}", e);
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_text_message(&text, &session, user_id, &tx).await {
                    error!("Error handling message: {}", e);
                }
            }
            Ok(Message::Binary(data)) => {
                debug!("Received binary message ({} bytes)", data.len());
            }
            Ok(Message::Ping(data)) => {
                let _ = tx.send(Message::Pong(data));
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong");
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket closed by client");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Cleanup
    broadcast_task.abort();
    send_task.abort();
    session.remove_user(user_id).await;
    info!(
        "WebSocket disconnected: session={}, user={} ({})",
        session_id.0, user_id.0, user_name
    );
}

async fn handle_text_message(
    text: &str,
    session: &Session,
    user_id: UserId,
    tx: &mpsc::UnboundedSender<Message>,
) -> anyhow::Result<()> {
    let msg: SyncMessage = serde_json::from_str(text)?;

    match msg {
        SyncMessage::Operation { operation } => {
            debug!(
                "Received operation from user {}: {:?}",
                user_id.0, operation.kind
            );
            session.add_operation(operation.clone()).await;

            // Send acknowledgment
            let ack = SyncMessage::OperationAck {
                operation_id: operation.id,
            };
            let json = serde_json::to_string(&ack)?;
            tx.send(Message::Text(json))?;
        }

        SyncMessage::SyncRequest { since } => {
            debug!("Sync request from user {}", user_id.0);
            let operations = session.get_operations_since(&since).await;

            // Calculate current vector clock
            let mut vector_clock = VectorClock::new();
            for op in &operations {
                vector_clock.increment(op.user_id);
            }

            let response = SyncMessage::SyncResponse {
                operations,
                vector_clock,
            };
            let json = serde_json::to_string(&response)?;
            tx.send(Message::Text(json))?;
        }

        SyncMessage::Presence { update } => {
            debug!("Presence update from user {}: {:?}", user_id.0, update);
            session.update_presence(user_id, update).await;
        }

        SyncMessage::Ping => {
            let pong = SyncMessage::Pong;
            let json = serde_json::to_string(&pong)?;
            tx.send(Message::Text(json))?;
        }

        SyncMessage::Pong => {
            debug!("Received pong from user {}", user_id.0);
        }

        _ => {
            warn!("Unexpected message type from user {}", user_id.0);
        }
    }

    Ok(())
}
