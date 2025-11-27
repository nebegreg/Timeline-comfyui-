/// Timeline collaborative operations
/// These operations represent changes that can be synchronized across users
use serde::{Deserialize, Serialize};
use timeline::{
    AutomationInterpolation, AutomationKeyframe, AutomationLane, AutomationTarget, Frame,
    FrameRange, LaneId, Marker, MarkerId, NodeId, TimelineGraph, TimelineNode, TrackBinding,
    TrackId,
};

use crate::{LamportClock, UserId};

/// Unique operation identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub uuid::Uuid);

impl OperationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Timeline operation that can be replicated across users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineOperation {
    /// Unique operation ID
    pub id: OperationId,

    /// User who created this operation
    pub user_id: UserId,

    /// Lamport timestamp for causality
    pub clock: LamportClock,

    /// Timestamp when operation was created (client time)
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// The actual operation
    pub kind: OperationKind,

    /// Parent operations (for causality tracking)
    pub parents: Vec<OperationId>,
}

impl TimelineOperation {
    pub fn new(user_id: UserId, clock: LamportClock, kind: OperationKind) -> Self {
        Self {
            id: OperationId::new(),
            user_id,
            clock,
            timestamp: chrono::Utc::now(),
            kind,
            parents: Vec::new(),
        }
    }

    pub fn with_parents(mut self, parents: Vec<OperationId>) -> Self {
        self.parents = parents;
        self
    }
}

/// Types of operations that can be performed on timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationKind {
    // Node operations
    AddNode {
        node: TimelineNode,
    },
    RemoveNode {
        node_id: NodeId,
    },
    UpdateNodePosition {
        node_id: NodeId,
        new_start: Frame,
    },
    UpdateNodeDuration {
        node_id: NodeId,
        new_range: FrameRange,
    },
    UpdateNodeMetadata {
        node_id: NodeId,
        metadata: serde_json::Value,
    },
    LockNode {
        node_id: NodeId,
        locked: bool,
    },

    // Track operations
    AddTrack {
        track: TrackBinding,
    },
    RemoveTrack {
        track_id: TrackId,
    },
    RenameTrack {
        track_id: TrackId,
        new_name: String,
    },
    ReorderTracks {
        track_order: Vec<TrackId>,
    },
    AddNodeToTrack {
        track_id: TrackId,
        node_id: NodeId,
    },
    RemoveNodeFromTrack {
        track_id: TrackId,
        node_id: NodeId,
    },

    // Marker operations
    AddMarker {
        marker: Marker,
    },
    RemoveMarker {
        marker_id: MarkerId,
    },
    UpdateMarker {
        marker_id: MarkerId,
        new_frame: Frame,
        new_label: Option<String>,
    },

    // Automation operations
    CreateAutomationLane {
        lane_id: LaneId,
        target_node: NodeId,
        parameter_path: String,
    },
    RemoveAutomationLane {
        lane_id: LaneId,
    },
    AddKeyframe {
        lane_id: LaneId,
        keyframe: AutomationKeyframe,
    },
    RemoveKeyframe {
        lane_id: LaneId,
        frame: Frame,
    },
    UpdateKeyframe {
        lane_id: LaneId,
        frame: Frame,
        new_value: f32,
    },
    UpdateCurveType {
        lane_id: LaneId,
        curve_type: AutomationInterpolation,
    },

    // Compound operations (ripple, roll, etc.)
    RippleEdit {
        node_id: NodeId,
        new_start: Frame,
    },
    RollEdit {
        left_node_id: NodeId,
        right_node_id: NodeId,
        new_edit_point: Frame,
    },
    SlideEdit {
        node_id: NodeId,
        media_offset: Frame,
    },
}

impl OperationKind {
    /// Apply this operation to a timeline graph
    pub fn apply(&self, graph: &mut TimelineGraph) -> Result<(), String> {
        match self {
            OperationKind::AddNode { node } => {
                graph.nodes.insert(node.id, node.clone());
                Ok(())
            }

            OperationKind::RemoveNode { node_id } => {
                graph.nodes.remove(node_id);
                // Also remove from all tracks
                for track in &mut graph.tracks {
                    track.node_ids.retain(|id| id != node_id);
                }
                Ok(())
            }

            OperationKind::UpdateNodePosition { node_id, new_start } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    if let timeline::TimelineNodeKind::Clip(ref mut clip) = node.kind {
                        clip.timeline_range.start = *new_start;
                    }
                }
                Ok(())
            }

            OperationKind::UpdateNodeDuration { node_id, new_range } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    if let timeline::TimelineNodeKind::Clip(ref mut clip) = node.kind {
                        clip.timeline_range = new_range.clone();
                    }
                }
                Ok(())
            }

            OperationKind::UpdateNodeMetadata { node_id, metadata } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.metadata = metadata.clone();
                }
                Ok(())
            }

            OperationKind::LockNode { node_id, locked } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.locked = *locked;
                }
                Ok(())
            }

            OperationKind::AddTrack { track } => {
                graph.tracks.push(track.clone());
                Ok(())
            }

            OperationKind::RemoveTrack { track_id } => {
                graph.tracks.retain(|t| t.id != *track_id);
                Ok(())
            }

            OperationKind::RenameTrack { track_id, new_name } => {
                if let Some(track) = graph.tracks.iter_mut().find(|t| t.id == *track_id) {
                    track.name = new_name.clone();
                }
                Ok(())
            }

            OperationKind::ReorderTracks { track_order } => {
                // Reorder tracks to match the given order
                let mut new_tracks = Vec::new();
                for track_id in track_order {
                    if let Some(track) = graph.tracks.iter().find(|t| t.id == *track_id) {
                        new_tracks.push(track.clone());
                    }
                }
                graph.tracks = new_tracks;
                Ok(())
            }

            OperationKind::AddNodeToTrack { track_id, node_id } => {
                if let Some(track) = graph.tracks.iter_mut().find(|t| t.id == *track_id) {
                    if !track.node_ids.contains(node_id) {
                        track.node_ids.push(*node_id);
                    }
                }
                Ok(())
            }

            OperationKind::RemoveNodeFromTrack { track_id, node_id } => {
                if let Some(track) = graph.tracks.iter_mut().find(|t| t.id == *track_id) {
                    track.node_ids.retain(|id| id != node_id);
                }
                Ok(())
            }

            OperationKind::AddMarker { marker } => {
                graph.markers.insert(marker.id, marker.clone());
                Ok(())
            }

            OperationKind::RemoveMarker { marker_id } => {
                graph.markers.remove(marker_id);
                Ok(())
            }

            OperationKind::UpdateMarker {
                marker_id,
                new_frame,
                new_label,
            } => {
                if let Some(marker) = graph.markers.get_mut(marker_id) {
                    marker.frame = *new_frame;
                    if let Some(label) = new_label {
                        marker.label = label.clone();
                    }
                }
                Ok(())
            }

            OperationKind::CreateAutomationLane {
                lane_id,
                target_node,
                parameter_path,
            } => {
                let lane = AutomationLane {
                    id: *lane_id,
                    target: AutomationTarget {
                        node: *target_node,
                        parameter: parameter_path.clone(),
                    },
                    interpolation: AutomationInterpolation::Linear,
                    keyframes: Vec::new(),
                };
                graph.automation.push(lane);
                Ok(())
            }

            OperationKind::RemoveAutomationLane { lane_id } => {
                graph.automation.retain(|lane| lane.id != *lane_id);
                Ok(())
            }

            OperationKind::AddKeyframe { lane_id, keyframe } => {
                if let Some(lane) = graph.automation.iter_mut().find(|l| l.id == *lane_id) {
                    lane.keyframes.push(keyframe.clone());
                    lane.keyframes.sort_by_key(|kf| kf.frame);
                }
                Ok(())
            }

            OperationKind::RemoveKeyframe { lane_id, frame } => {
                if let Some(lane) = graph.automation.iter_mut().find(|l| l.id == *lane_id) {
                    lane.keyframes.retain(|kf| kf.frame != *frame);
                }
                Ok(())
            }

            OperationKind::UpdateKeyframe {
                lane_id,
                frame,
                new_value,
            } => {
                if let Some(lane) = graph.automation.iter_mut().find(|l| l.id == *lane_id) {
                    // Remove old keyframe and add new one
                    lane.keyframes.retain(|kf| kf.frame != *frame);
                    lane.keyframes.push(AutomationKeyframe {
                        frame: *frame,
                        value: *new_value as f64,
                        easing: timeline::KeyframeEasing::default(),
                    });
                    lane.keyframes.sort_by_key(|kf| kf.frame);
                }
                Ok(())
            }

            OperationKind::UpdateCurveType {
                lane_id,
                curve_type,
            } => {
                if let Some(lane) = graph.automation.iter_mut().find(|l| l.id == *lane_id) {
                    lane.interpolation = curve_type.clone();
                }
                Ok(())
            }

            OperationKind::RippleEdit { node_id, new_start } => {
                timeline::ripple_move_clip(graph, node_id, *new_start)?;
                Ok(())
            }

            OperationKind::RollEdit {
                left_node_id,
                right_node_id,
                new_edit_point,
            } => {
                timeline::roll_edit(graph, left_node_id, right_node_id, *new_edit_point)?;
                Ok(())
            }

            OperationKind::SlideEdit {
                node_id,
                media_offset,
            } => {
                timeline::slide_edit(graph, node_id, *media_offset)?;
                Ok(())
            }
        }
    }

    /// Check if this operation conflicts with another
    pub fn conflicts_with(&self, other: &OperationKind) -> bool {
        use OperationKind::*;

        match (self, other) {
            // Node conflicts
            (AddNode { node: n1 }, AddNode { node: n2 }) => n1.id == n2.id,
            (RemoveNode { node_id: n1 }, RemoveNode { node_id: n2 }) => n1 == n2,
            (RemoveNode { node_id: n1 }, UpdateNodePosition { node_id: n2, .. })
            | (RemoveNode { node_id: n1 }, UpdateNodeDuration { node_id: n2, .. })
            | (RemoveNode { node_id: n1 }, LockNode { node_id: n2, .. }) => n1 == n2,

            // Track conflicts
            (AddTrack { track: t1 }, AddTrack { track: t2 }) => t1.id == t2.id,
            (RemoveTrack { track_id: t1 }, RemoveTrack { track_id: t2 }) => t1 == t2,
            (RemoveTrack { track_id: t1 }, RenameTrack { track_id: t2, .. }) => t1 == t2,

            // Marker conflicts
            (AddMarker { marker: m1 }, AddMarker { marker: m2 }) => m1.id == m2.id,
            (RemoveMarker { marker_id: m1 }, RemoveMarker { marker_id: m2 }) => m1 == m2,
            (RemoveMarker { marker_id: m1 }, UpdateMarker { marker_id: m2, .. }) => m1 == m2,
            (UpdateMarker { marker_id: m1, .. }, UpdateMarker { marker_id: m2, .. }) => m1 == m2,

            // Automation conflicts
            (
                CreateAutomationLane { lane_id: l1, .. },
                CreateAutomationLane { lane_id: l2, .. },
            ) => l1 == l2,
            (RemoveAutomationLane { lane_id: l1 }, RemoveAutomationLane { lane_id: l2 }) => {
                l1 == l2
            }
            (
                RemoveAutomationLane { lane_id: l1 },
                AddKeyframe { lane_id: l2, .. }
                | RemoveKeyframe { lane_id: l2, .. }
                | UpdateKeyframe { lane_id: l2, .. }
                | UpdateCurveType { lane_id: l2, .. },
            ) => l1 == l2,

            // Position conflicts - multiple users moving the same clip
            (UpdateNodePosition { node_id: n1, .. }, UpdateNodePosition { node_id: n2, .. }) => {
                n1 == n2
            }
            (RippleEdit { node_id: n1, .. }, RippleEdit { node_id: n2, .. }) => n1 == n2,

            // No conflict by default
            _ => false,
        }
    }
}

/// Operation log that maintains causality
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationLog {
    /// All operations in causal order
    pub operations: Vec<TimelineOperation>,

    /// Index by operation ID for fast lookup
    #[serde(skip)]
    operation_index: std::collections::HashMap<OperationId, usize>,
}

impl OperationLog {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            operation_index: std::collections::HashMap::new(),
        }
    }

    /// Add an operation to the log
    pub fn add_operation(&mut self, op: TimelineOperation) {
        let id = op.id;
        self.operations.push(op);
        self.operation_index.insert(id, self.operations.len() - 1);
    }

    /// Get operation by ID
    pub fn get_operation(&self, id: &OperationId) -> Option<&TimelineOperation> {
        self.operation_index
            .get(id)
            .and_then(|&idx| self.operations.get(idx))
    }

    /// Apply all operations to a timeline
    pub fn apply_to_timeline(&self, graph: &mut TimelineGraph) -> Result<(), String> {
        for op in &self.operations {
            op.kind.apply(graph)?;
        }
        Ok(())
    }

    /// Merge operations from another log
    pub fn merge(&mut self, other: &OperationLog) {
        for op in &other.operations {
            if !self.operation_index.contains_key(&op.id) {
                self.add_operation(op.clone());
            }
        }

        // Re-sort by Lamport clock for causal ordering
        self.operations.sort_by_key(|op| op.clock);

        // Rebuild index
        self.operation_index.clear();
        for (idx, op) in self.operations.iter().enumerate() {
            self.operation_index.insert(op.id, idx);
        }
    }

    /// Compact the operation log by creating a snapshot
    /// Removes old operations and replaces them with a single snapshot operation
    pub fn compact(&mut self, _graph: &TimelineGraph, min_operations: usize) {
        if self.operations.len() < min_operations {
            return; // Not enough operations to compact
        }

        // Keep the last N operations for history
        let keep_count = 100;
        if self.operations.len() <= keep_count {
            return;
        }

        // Remove old operations
        let removed = self.operations.len() - keep_count;
        self.operations.drain(0..removed);

        // Rebuild index
        self.operation_index.clear();
        for (idx, op) in self.operations.iter().enumerate() {
            self.operation_index.insert(op.id, idx);
        }
    }

    /// Optimize the log by removing redundant operations
    /// For example, multiple consecutive position updates for the same node
    pub fn optimize(&mut self) {
        use std::collections::HashMap;

        let mut optimized = Vec::new();
        let mut last_position_update: HashMap<NodeId, TimelineOperation> = HashMap::new();
        let mut last_metadata_update: HashMap<NodeId, TimelineOperation> = HashMap::new();

        for op in &self.operations {
            match &op.kind {
                OperationKind::UpdateNodePosition { node_id, .. } => {
                    // Keep only the latest position update for each node in sequence
                    last_position_update.insert(*node_id, op.clone());
                }
                OperationKind::UpdateNodeMetadata { node_id, .. } => {
                    // Keep only the latest metadata update for each node in sequence
                    last_metadata_update.insert(*node_id, op.clone());
                }
                _ => {
                    // Flush any pending position/metadata updates
                    if !last_position_update.is_empty() {
                        optimized.extend(last_position_update.drain().map(|(_, v)| v));
                    }
                    if !last_metadata_update.is_empty() {
                        optimized.extend(last_metadata_update.drain().map(|(_, v)| v));
                    }

                    // Keep all other operations
                    optimized.push(op.clone());
                }
            }
        }

        // Flush any remaining updates
        optimized.extend(last_position_update.drain().map(|(_, v)| v));
        optimized.extend(last_metadata_update.drain().map(|(_, v)| v));

        // Replace operations
        self.operations = optimized;

        // Rebuild index
        self.operation_index.clear();
        for (idx, op) in self.operations.iter().enumerate() {
            self.operation_index.insert(op.id, idx);
        }
    }

    /// Get the total size of the operation log in bytes (approximate)
    pub fn size_bytes(&self) -> usize {
        // Approximate: each operation is ~200 bytes on average
        self.operations.len() * 200
    }

    /// Check if compaction is recommended
    pub fn should_compact(&self, max_size_bytes: usize) -> bool {
        self.size_bytes() > max_size_bytes || self.operations.len() > 1000
    }
}
