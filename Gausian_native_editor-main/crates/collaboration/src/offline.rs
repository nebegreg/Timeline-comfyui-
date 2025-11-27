/// Offline support for collaborative editing
/// Queue operations when offline and sync when reconnected
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::{SessionId, TimelineOperation, UserId, VectorClock};

/// Offline operation queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueue {
    /// Session this queue belongs to
    pub session_id: SessionId,

    /// User ID
    pub user_id: UserId,

    /// Queued operations (not yet sent to server)
    pub pending_operations: Vec<TimelineOperation>,

    /// Last known vector clock (before going offline)
    pub last_known_clock: VectorClock,

    /// Queue creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OfflineQueue {
    pub fn new(session_id: SessionId, user_id: UserId) -> Self {
        Self {
            session_id,
            user_id,
            pending_operations: Vec::new(),
            last_known_clock: VectorClock::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Add an operation to the queue
    pub fn enqueue(&mut self, operation: TimelineOperation) {
        self.pending_operations.push(operation);
    }

    /// Get all pending operations
    pub fn drain_pending(&mut self) -> Vec<TimelineOperation> {
        std::mem::take(&mut self.pending_operations)
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.pending_operations.is_empty()
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        self.pending_operations.len()
    }
}

/// Offline queue manager with persistent storage
pub struct OfflineQueueManager {
    /// Storage directory for queues
    storage_dir: PathBuf,

    /// Active queue
    current_queue: Option<OfflineQueue>,
}

impl OfflineQueueManager {
    /// Create a new queue manager
    pub fn new(storage_dir: impl AsRef<Path>) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            current_queue: None,
        }
    }

    /// Start a new offline queue
    pub fn start_queue(&mut self, session_id: SessionId, user_id: UserId) {
        self.current_queue = Some(OfflineQueue::new(session_id, user_id));
    }

    /// Add operation to current queue
    pub fn enqueue(&mut self, operation: TimelineOperation) -> Result<(), String> {
        if let Some(queue) = &mut self.current_queue {
            queue.enqueue(operation);
            Ok(())
        } else {
            Err("No active offline queue".to_string())
        }
    }

    /// Get pending operations and clear queue
    pub fn drain_pending(&mut self) -> Vec<TimelineOperation> {
        if let Some(queue) = &mut self.current_queue {
            queue.drain_pending()
        } else {
            Vec::new()
        }
    }

    /// Save current queue to disk
    pub async fn save_queue(&self) -> Result<(), String> {
        if let Some(queue) = &self.current_queue {
            if queue.is_empty() {
                return Ok(());
            }

            // Create storage directory if it doesn't exist
            fs::create_dir_all(&self.storage_dir)
                .await
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;

            // Save queue as JSON
            let filename = format!("offline_queue_{}.json", queue.session_id.0);
            let filepath = self.storage_dir.join(filename);

            let json = serde_json::to_string_pretty(queue)
                .map_err(|e| format!("Failed to serialize queue: {}", e))?;

            fs::write(&filepath, json)
                .await
                .map_err(|e| format!("Failed to write queue to disk: {}", e))?;

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Load queue from disk
    pub async fn load_queue(&mut self, session_id: SessionId) -> Result<(), String> {
        let filename = format!("offline_queue_{}.json", session_id.0);
        let filepath = self.storage_dir.join(filename);

        if !filepath.exists() {
            return Ok(());
        }

        let json = fs::read_to_string(&filepath)
            .await
            .map_err(|e| format!("Failed to read queue from disk: {}", e))?;

        let queue: OfflineQueue = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize queue: {}", e))?;

        self.current_queue = Some(queue);

        Ok(())
    }

    /// Delete saved queue from disk
    pub async fn delete_saved_queue(&self, session_id: SessionId) -> Result<(), String> {
        let filename = format!("offline_queue_{}.json", session_id.0);
        let filepath = self.storage_dir.join(filename);

        if filepath.exists() {
            fs::remove_file(&filepath)
                .await
                .map_err(|e| format!("Failed to delete queue file: {}", e))?;
        }

        Ok(())
    }

    /// Clear current queue
    pub fn clear_queue(&mut self) {
        self.current_queue = None;
    }

    /// Check if there are pending operations
    pub fn has_pending(&self) -> bool {
        self.current_queue
            .as_ref()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Get number of pending operations
    pub fn pending_count(&self) -> usize {
        self.current_queue.as_ref().map(|q| q.len()).unwrap_or(0)
    }
}

/// Offline sync strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStrategy {
    /// Send all pending operations immediately
    Immediate,

    /// Batch operations and send in chunks
    Batched { batch_size: usize },

    /// Merge compatible operations before sending
    Optimized,
}

impl Default for SyncStrategy {
    fn default() -> Self {
        Self::Optimized
    }
}

/// Handle reconnection and sync
pub async fn sync_offline_operations(
    queue_manager: &mut OfflineQueueManager,
    send_operation: impl Fn(TimelineOperation) -> Result<(), String>,
    strategy: SyncStrategy,
) -> Result<usize, String> {
    let pending_ops = queue_manager.drain_pending();

    if pending_ops.is_empty() {
        return Ok(0);
    }

    let ops_to_send = match strategy {
        SyncStrategy::Immediate => pending_ops,

        SyncStrategy::Batched { batch_size } => {
            // Send in batches
            let mut sent = 0;
            for chunk in pending_ops.chunks(batch_size) {
                for op in chunk {
                    send_operation(op.clone())?;
                    sent += 1;
                }
                // Could add delay between batches if needed
            }
            return Ok(sent);
        }

        SyncStrategy::Optimized => {
            // Merge compatible operations
            optimize_operations(pending_ops)
        }
    };

    let count = ops_to_send.len();

    for op in ops_to_send {
        send_operation(op)?;
    }

    Ok(count)
}

/// Optimize operations by merging compatible ones
fn optimize_operations(operations: Vec<TimelineOperation>) -> Vec<TimelineOperation> {
    // Simple optimization: remove redundant operations
    // For example, if we have multiple position updates for the same node,
    // keep only the latest one

    use crate::OperationKind;
    use std::collections::HashMap;

    let mut optimized = Vec::new();
    let mut position_updates: HashMap<timeline::NodeId, TimelineOperation> = HashMap::new();

    for op in operations {
        match &op.kind {
            OperationKind::UpdateNodePosition { node_id, .. } => {
                // Keep only the latest position update for each node
                position_updates.insert(*node_id, op);
            }
            _ => {
                // Keep all other operations
                optimized.push(op);
            }
        }
    }

    // Add the latest position updates
    optimized.extend(position_updates.into_values());

    // Sort by clock to maintain causality
    optimized.sort_by_key(|op| op.clock);

    optimized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_queue() {
        let session_id = SessionId::new();
        let user_id = UserId::new();
        let mut queue = OfflineQueue::new(session_id, user_id);

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        // Add operation
        let op = TimelineOperation::new(
            user_id,
            crate::LamportClock::new(),
            crate::OperationKind::AddMarker {
                marker: timeline::Marker::new(100, "Test".to_string()),
            },
        );

        queue.enqueue(op);
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);

        // Drain
        let ops = queue.drain_pending();
        assert_eq!(ops.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_manager() {
        let mut manager = OfflineQueueManager::new("/tmp/collab_test");
        let session_id = SessionId::new();
        let user_id = UserId::new();

        assert!(!manager.has_pending());

        manager.start_queue(session_id, user_id);

        let op = TimelineOperation::new(
            user_id,
            crate::LamportClock::new(),
            crate::OperationKind::AddMarker {
                marker: timeline::Marker::new(100, "Test".to_string()),
            },
        );

        manager.enqueue(op).unwrap();
        assert!(manager.has_pending());
        assert_eq!(manager.pending_count(), 1);

        let ops = manager.drain_pending();
        assert_eq!(ops.len(), 1);
        assert!(!manager.has_pending());
    }
}
