/// Conflict detection and resolution for collaborative editing
use serde::{Deserialize, Serialize};
use timeline::NodeId;

use crate::{OperationKind, TimelineOperation, UserId};

/// Represents a conflict between two operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// The two conflicting operations
    pub op1: TimelineOperation,
    pub op2: TimelineOperation,

    /// Type of conflict
    pub kind: ConflictKind,

    /// When the conflict was detected
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Types of conflicts that can occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictKind {
    /// Two users tried to delete the same node
    DuplicateDelete { node_id: NodeId },

    /// Two users moved the same node to different positions
    ConcurrentMove {
        node_id: NodeId,
        position1: i64,
        position2: i64,
    },

    /// Two users edited the same node property differently
    PropertyConflict { node_id: NodeId, property: String },

    /// Two users created nodes with the same ID (shouldn't happen with UUIDs)
    DuplicateCreate { node_id: NodeId },

    /// General operation conflict
    OperationConflict { description: String },
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Last Write Wins - use the operation with higher timestamp
    LastWriteWins,

    /// User Priority - prefer operations from specific users
    UserPriority,

    /// Manual - require manual resolution
    Manual,

    /// Merge - try to merge both operations
    Merge,
}

/// Result of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionResult {
    /// Use first operation, discard second
    UseFirst,

    /// Use second operation, discard first
    UseSecond,

    /// Keep both operations (no actual conflict)
    KeepBoth,

    /// Requires manual resolution
    RequiresManual { conflict: Conflict },

    /// Create a merged operation
    Merged { merged_op: TimelineOperation },
}

/// Conflict resolver
pub struct ConflictResolver {
    strategy: ResolutionStrategy,
    priority_users: Vec<UserId>,
}

impl ConflictResolver {
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            strategy,
            priority_users: Vec::new(),
        }
    }

    pub fn with_priority_users(mut self, users: Vec<UserId>) -> Self {
        self.priority_users = users;
        self
    }

    /// Detect if two operations conflict
    pub fn detect_conflict(
        &self,
        op1: &TimelineOperation,
        op2: &TimelineOperation,
    ) -> Option<Conflict> {
        if !op1.kind.conflicts_with(&op2.kind) {
            return None;
        }

        let kind = self.classify_conflict(&op1.kind, &op2.kind)?;

        Some(Conflict {
            op1: op1.clone(),
            op2: op2.clone(),
            kind,
            detected_at: chrono::Utc::now(),
        })
    }

    /// Classify the type of conflict
    fn classify_conflict(
        &self,
        kind1: &OperationKind,
        kind2: &OperationKind,
    ) -> Option<ConflictKind> {
        use OperationKind::*;

        match (kind1, kind2) {
            (RemoveNode { node_id: n1 }, RemoveNode { node_id: n2 }) if n1 == n2 => {
                Some(ConflictKind::DuplicateDelete { node_id: *n1 })
            }

            (
                UpdateNodePosition {
                    node_id: n1,
                    new_start: p1,
                },
                UpdateNodePosition {
                    node_id: n2,
                    new_start: p2,
                },
            ) if n1 == n2 => Some(ConflictKind::ConcurrentMove {
                node_id: *n1,
                position1: *p1,
                position2: *p2,
            }),

            (AddNode { node: n1 }, AddNode { node: n2 }) if n1.id == n2.id => {
                Some(ConflictKind::DuplicateCreate { node_id: n1.id })
            }

            (
                UpdateNodeMetadata {
                    node_id: n1,
                    metadata: _,
                },
                UpdateNodeMetadata {
                    node_id: n2,
                    metadata: _,
                },
            ) if n1 == n2 => Some(ConflictKind::PropertyConflict {
                node_id: *n1,
                property: "metadata".to_string(),
            }),

            _ => Some(ConflictKind::OperationConflict {
                description: format!("Conflict between {:?} and {:?}", kind1, kind2),
            }),
        }
    }

    /// Resolve a conflict using the configured strategy
    pub fn resolve(&self, conflict: &Conflict) -> ResolutionResult {
        match self.strategy {
            ResolutionStrategy::LastWriteWins => self.resolve_last_write_wins(conflict),
            ResolutionStrategy::UserPriority => self.resolve_user_priority(conflict),
            ResolutionStrategy::Manual => ResolutionResult::RequiresManual {
                conflict: conflict.clone(),
            },
            ResolutionStrategy::Merge => self.resolve_merge(conflict),
        }
    }

    /// Resolve using last-write-wins strategy
    fn resolve_last_write_wins(&self, conflict: &Conflict) -> ResolutionResult {
        // Use Lamport clock for ordering
        if conflict.op1.clock > conflict.op2.clock {
            ResolutionResult::UseFirst
        } else if conflict.op2.clock > conflict.op1.clock {
            ResolutionResult::UseSecond
        } else {
            // Clocks are equal, use timestamp
            if conflict.op1.timestamp > conflict.op2.timestamp {
                ResolutionResult::UseFirst
            } else {
                ResolutionResult::UseSecond
            }
        }
    }

    /// Resolve using user priority
    fn resolve_user_priority(&self, conflict: &Conflict) -> ResolutionResult {
        let user1_priority = self
            .priority_users
            .iter()
            .position(|u| *u == conflict.op1.user_id);
        let user2_priority = self
            .priority_users
            .iter()
            .position(|u| *u == conflict.op2.user_id);

        match (user1_priority, user2_priority) {
            (Some(p1), Some(p2)) => {
                if p1 < p2 {
                    ResolutionResult::UseFirst
                } else {
                    ResolutionResult::UseSecond
                }
            }
            (Some(_), None) => ResolutionResult::UseFirst,
            (None, Some(_)) => ResolutionResult::UseSecond,
            (None, None) => self.resolve_last_write_wins(conflict),
        }
    }

    /// Attempt to merge conflicting operations
    fn resolve_merge(&self, conflict: &Conflict) -> ResolutionResult {
        use ConflictKind::*;

        match &conflict.kind {
            // For duplicate deletes, both operations achieve the same result
            DuplicateDelete { .. } => ResolutionResult::UseFirst,

            // For concurrent moves, use last-write-wins
            ConcurrentMove { .. } => self.resolve_last_write_wins(conflict),

            // For property conflicts, try to merge if possible
            PropertyConflict { .. } => {
                // For now, fall back to last-write-wins
                // In the future, we could implement intelligent merging
                self.resolve_last_write_wins(conflict)
            }

            // Duplicate creates should never happen with UUIDs
            DuplicateCreate { .. } => ResolutionResult::UseFirst,

            // For general conflicts, require manual resolution
            OperationConflict { .. } => ResolutionResult::RequiresManual {
                conflict: conflict.clone(),
            },
        }
    }
}

/// Conflict manager tracks and resolves conflicts
pub struct ConflictManager {
    resolver: ConflictResolver,
    pending_conflicts: Vec<Conflict>,
}

impl ConflictManager {
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            resolver: ConflictResolver::new(strategy),
            pending_conflicts: Vec::new(),
        }
    }

    /// Check for conflicts between operations
    pub fn check_conflicts(
        &mut self,
        op: &TimelineOperation,
        other_ops: &[TimelineOperation],
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for other_op in other_ops {
            // Skip if operations are from same user
            if op.user_id == other_op.user_id {
                continue;
            }

            // Skip if there's a clear causal relationship
            if op.parents.contains(&other_op.id) || other_op.parents.contains(&op.id) {
                continue;
            }

            if let Some(conflict) = self.resolver.detect_conflict(op, other_op) {
                conflicts.push(conflict);
            }
        }

        conflicts
    }

    /// Resolve a conflict
    pub fn resolve_conflict(&self, conflict: &Conflict) -> ResolutionResult {
        self.resolver.resolve(conflict)
    }

    /// Add a conflict for manual resolution
    pub fn add_pending_conflict(&mut self, conflict: Conflict) {
        self.pending_conflicts.push(conflict);
    }

    /// Get all pending conflicts
    pub fn get_pending_conflicts(&self) -> &[Conflict] {
        &self.pending_conflicts
    }

    /// Clear pending conflicts
    pub fn clear_pending_conflicts(&mut self) {
        self.pending_conflicts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LamportClock, SessionId};
    use timeline::{ClipNode, Frame, FrameRange, NodeId, TimelineNode, TimelineNodeKind};

    fn create_test_clip(start: Frame, duration: Frame) -> TimelineNode {
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
    fn test_duplicate_delete_conflict() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins);

        let node_id = NodeId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let op1 = TimelineOperation::new(
            user1,
            LamportClock(1),
            OperationKind::RemoveNode { node_id },
        );

        let op2 = TimelineOperation::new(
            user2,
            LamportClock(2),
            OperationKind::RemoveNode { node_id },
        );

        let conflict = resolver.detect_conflict(&op1, &op2);
        assert!(conflict.is_some());

        let conflict = conflict.unwrap();
        assert!(matches!(
            conflict.kind,
            ConflictKind::DuplicateDelete { .. }
        ));
    }

    #[test]
    fn test_concurrent_move_conflict() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins);

        let node_id = NodeId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let op1 = TimelineOperation::new(
            user1,
            LamportClock(1),
            OperationKind::UpdateNodePosition {
                node_id,
                new_start: 100,
            },
        );

        let op2 = TimelineOperation::new(
            user2,
            LamportClock(2),
            OperationKind::UpdateNodePosition {
                node_id,
                new_start: 200,
            },
        );

        let conflict = resolver.detect_conflict(&op1, &op2);
        assert!(conflict.is_some());

        let conflict = conflict.unwrap();
        assert!(matches!(conflict.kind, ConflictKind::ConcurrentMove { .. }));
    }

    #[test]
    fn test_last_write_wins_resolution() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins);

        let node_id = NodeId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let op1 = TimelineOperation::new(
            user1,
            LamportClock(1),
            OperationKind::UpdateNodePosition {
                node_id,
                new_start: 100,
            },
        );

        let op2 = TimelineOperation::new(
            user2,
            LamportClock(2),
            OperationKind::UpdateNodePosition {
                node_id,
                new_start: 200,
            },
        );

        let conflict = resolver.detect_conflict(&op1, &op2).unwrap();
        let resolution = resolver.resolve(&conflict);

        // op2 has higher clock, should win
        assert!(matches!(resolution, ResolutionResult::UseSecond));
    }
}
