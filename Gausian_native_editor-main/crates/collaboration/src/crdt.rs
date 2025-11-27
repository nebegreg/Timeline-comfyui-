/// CRDT (Conflict-free Replicated Data Type) implementation for timeline
/// Ensures eventual consistency across distributed collaborators
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use timeline::TimelineGraph;

use crate::{LamportClock, OperationId, OperationLog, SessionId, TimelineOperation, UserId};

/// CRDT document representing a collaborative timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTTimeline {
    /// Session this timeline belongs to
    pub session_id: SessionId,

    /// Current user editing this timeline
    pub user_id: UserId,

    /// Lamport clock for this replica
    pub clock: LamportClock,

    /// Operation log (operation-based CRDT)
    pub operation_log: OperationLog,

    /// Current timeline state (materialized view)
    pub timeline: TimelineGraph,

    /// Vector clock for tracking causal dependencies
    pub vector_clock: VectorClock,

    /// Operations received but not yet applied (waiting for dependencies)
    pending_operations: Vec<TimelineOperation>,
}

impl CRDTTimeline {
    pub fn new(session_id: SessionId, user_id: UserId) -> Self {
        Self {
            session_id,
            user_id,
            clock: LamportClock::new(),
            operation_log: OperationLog::new(),
            timeline: TimelineGraph::default(),
            vector_clock: VectorClock::new(),
            pending_operations: Vec::new(),
        }
    }

    /// Apply a local operation (created by this user)
    pub fn apply_local_operation(
        &mut self,
        kind: crate::OperationKind,
    ) -> Result<TimelineOperation, String> {
        // Increment clock
        self.clock.tick();

        // Create operation with current clock
        let parents = self.get_latest_operation_ids();
        let op = TimelineOperation::new(self.user_id, self.clock, kind).with_parents(parents);

        // Apply to timeline
        op.kind.apply(&mut self.timeline)?;

        // Add to operation log
        self.operation_log.add_operation(op.clone());

        // Update vector clock
        self.vector_clock.increment(self.user_id);

        Ok(op)
    }

    /// Apply a remote operation (received from another user)
    pub fn apply_remote_operation(&mut self, op: TimelineOperation) -> Result<(), String> {
        // Update our clock
        self.clock.update(op.clock);

        // Check if we have all parent operations
        if !self.has_all_parents(&op) {
            // Queue for later
            self.pending_operations.push(op);
            return Ok(());
        }

        // Apply to timeline
        op.kind.apply(&mut self.timeline)?;

        // Add to operation log
        self.operation_log.add_operation(op.clone());

        // Update vector clock
        self.vector_clock.increment(op.user_id);

        // Try to apply any pending operations that are now ready
        self.apply_pending_operations()?;

        Ok(())
    }

    /// Merge state from another replica
    pub fn merge(&mut self, other: &CRDTTimeline) -> Result<(), String> {
        // Merge operation logs
        self.operation_log.merge(&other.operation_log);

        // Update clock
        self.clock.update(other.clock);

        // Merge vector clocks
        self.vector_clock.merge(&other.vector_clock);

        // Rebuild timeline from merged operations
        self.rebuild_timeline()?;

        Ok(())
    }

    /// Rebuild timeline state from operation log
    fn rebuild_timeline(&mut self) -> Result<(), String> {
        self.timeline = TimelineGraph::default();
        self.operation_log.apply_to_timeline(&mut self.timeline)?;
        Ok(())
    }

    /// Check if we have all parent operations
    fn has_all_parents(&self, op: &TimelineOperation) -> bool {
        op.parents
            .iter()
            .all(|parent_id| self.operation_log.get_operation(parent_id).is_some())
    }

    /// Apply pending operations that are now ready
    fn apply_pending_operations(&mut self) -> Result<(), String> {
        let mut applied_any = true;

        while applied_any {
            applied_any = false;

            // Collect pending operations to avoid borrow conflicts
            let pending: Vec<_> = self.pending_operations.drain(..).collect();
            let mut remaining = Vec::new();

            for op in pending {
                if self.has_all_parents(&op) {
                    op.kind.apply(&mut self.timeline)?;
                    self.operation_log.add_operation(op.clone());
                    self.vector_clock.increment(op.user_id);
                    applied_any = true;
                } else {
                    remaining.push(op);
                }
            }

            self.pending_operations = remaining;
        }

        Ok(())
    }

    /// Get IDs of the latest operations (for setting as parents)
    fn get_latest_operation_ids(&self) -> Vec<OperationId> {
        // For simplicity, use the last N operations
        // In a real implementation, we'd track the frontier (operations with no children)
        self.operation_log
            .operations
            .iter()
            .rev()
            .take(5)
            .map(|op| op.id)
            .collect()
    }

    /// Get all operations since a given vector clock
    pub fn get_operations_since(&self, since: &VectorClock) -> Vec<TimelineOperation> {
        self.operation_log
            .operations
            .iter()
            .filter(|op| {
                let op_version = self.vector_clock.get(op.user_id);
                let since_version = since.get(op.user_id);
                op_version > since_version
            })
            .cloned()
            .collect()
    }

    /// Get the current vector clock (for sync)
    pub fn get_vector_clock(&self) -> &VectorClock {
        &self.vector_clock
    }
}

/// Vector clock for tracking causal dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorClock {
    /// Map of user ID to their operation count
    clocks: HashMap<UserId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// Increment the clock for a user
    pub fn increment(&mut self, user_id: UserId) {
        *self.clocks.entry(user_id).or_insert(0) += 1;
    }

    /// Get the current value for a user
    pub fn get(&self, user_id: UserId) -> u64 {
        *self.clocks.get(&user_id).unwrap_or(&0)
    }

    /// Merge with another vector clock (take max)
    pub fn merge(&mut self, other: &VectorClock) {
        for (user_id, &count) in &other.clocks {
            let current = self.clocks.entry(*user_id).or_insert(0);
            *current = (*current).max(count);
        }
    }

    /// Check if this clock is concurrent with another
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        let users: HashSet<_> = self.clocks.keys().chain(other.clocks.keys()).collect();

        let mut less = false;
        let mut greater = false;

        for user_id in users {
            let self_val = self.get(*user_id);
            let other_val = other.get(*user_id);

            if self_val < other_val {
                less = true;
            }
            if self_val > other_val {
                greater = true;
            }
        }

        less && greater
    }

    /// Check if this clock happened before another
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let users: HashSet<_> = self.clocks.keys().chain(other.clocks.keys()).collect();

        let mut any_less = false;

        for user_id in users {
            let self_val = self.get(*user_id);
            let other_val = other.get(*user_id);

            if self_val > other_val {
                return false;
            }
            if self_val < other_val {
                any_less = true;
            }
        }

        any_less
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OperationKind;
    use timeline::{
        Frame, FrameRange, NodeId, TimelineNode, TimelineNodeKind, TrackBinding, TrackKind,
    };

    fn create_test_clip(start: Frame, duration: Frame) -> TimelineNode {
        use timeline::ClipNode;
        TimelineNode {
            id: NodeId::new(),
            label: Some("Test Clip".to_string()),
            kind: TimelineNodeKind::Clip(ClipNode {
                asset_id: Some("test.mp4".to_string()),
                media_range: FrameRange {
                    start: 0,
                    duration: duration * 3,
                },
                timeline_range: FrameRange { start, duration },
                playback_rate: 1.0,
                reverse: false,
                metadata: serde_json::Value::Null,
            }),
            locked: false,
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_local_operation() {
        let session_id = SessionId::new();
        let user_id = UserId::new();
        let mut crdt = CRDTTimeline::new(session_id, user_id);

        // Add a node
        let node = create_test_clip(0, 100);
        let node_id = node.id;

        let op = crdt
            .apply_local_operation(OperationKind::AddNode { node })
            .unwrap();

        // Check operation was created
        assert_eq!(op.user_id, user_id);
        assert_eq!(op.clock.0, 1);

        // Check node was added to timeline
        assert!(crdt.timeline.nodes.contains_key(&node_id));
    }

    #[test]
    fn test_remote_operation() {
        let session_id = SessionId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let mut crdt1 = CRDTTimeline::new(session_id, user1);
        let mut crdt2 = CRDTTimeline::new(session_id, user2);

        // User 1 adds a node
        let node = create_test_clip(0, 100);
        let node_id = node.id;
        let op = crdt1
            .apply_local_operation(OperationKind::AddNode { node })
            .unwrap();

        // User 2 receives the operation
        crdt2.apply_remote_operation(op).unwrap();

        // Check node exists in both timelines
        assert!(crdt1.timeline.nodes.contains_key(&node_id));
        assert!(crdt2.timeline.nodes.contains_key(&node_id));
    }

    #[test]
    fn test_merge() {
        let session_id = SessionId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let mut crdt1 = CRDTTimeline::new(session_id, user1);
        let mut crdt2 = CRDTTimeline::new(session_id, user2);

        // User 1 adds a node
        let node1 = create_test_clip(0, 100);
        let node1_id = node1.id;
        crdt1
            .apply_local_operation(OperationKind::AddNode { node: node1 })
            .unwrap();

        // User 2 adds a different node
        let node2 = create_test_clip(100, 100);
        let node2_id = node2.id;
        crdt2
            .apply_local_operation(OperationKind::AddNode { node: node2 })
            .unwrap();

        // Merge crdt2 into crdt1
        crdt1.merge(&crdt2).unwrap();

        // Both nodes should exist
        assert!(crdt1.timeline.nodes.contains_key(&node1_id));
        assert!(crdt1.timeline.nodes.contains_key(&node2_id));
    }

    #[test]
    fn test_vector_clock() {
        let user1 = UserId::new();
        let user2 = UserId::new();

        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        // User 1 makes 3 operations
        vc1.increment(user1);
        vc1.increment(user1);
        vc1.increment(user1);

        // User 2 makes 2 operations
        vc2.increment(user2);
        vc2.increment(user2);

        // They are concurrent
        assert!(vc1.is_concurrent(&vc2));

        // Merge
        vc1.merge(&vc2);

        // Now vc1 has both
        assert_eq!(vc1.get(user1), 3);
        assert_eq!(vc1.get(user2), 2);
    }
}
