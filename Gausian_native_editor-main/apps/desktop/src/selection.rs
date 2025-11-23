/// Multi-clip selection system for timeline
/// Phase 1: Timeline Polish & UX Improvements

use std::collections::HashSet;
use timeline::{NodeId, TrackId};

#[derive(Clone, Debug, Default)]
pub struct SelectionState {
    /// Set of selected node IDs
    pub selected_nodes: HashSet<NodeId>,
    /// Primary selection (for operations requiring a single target)
    pub primary_node: Option<NodeId>,
    /// Track-level selection (for track operations)
    pub selected_tracks: HashSet<TrackId>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a single node, clearing previous selection
    pub fn select_single(&mut self, node_id: NodeId) {
        self.selected_nodes.clear();
        self.selected_nodes.insert(node_id);
        self.primary_node = Some(node_id);
    }

    /// Add node to selection (Shift-click)
    pub fn add_to_selection(&mut self, node_id: NodeId) {
        self.selected_nodes.insert(node_id);
        if self.primary_node.is_none() {
            self.primary_node = Some(node_id);
        }
    }

    /// Remove node from selection
    pub fn remove_from_selection(&mut self, node_id: NodeId) {
        self.selected_nodes.remove(&node_id);
        if self.primary_node == Some(node_id) {
            self.primary_node = self.selected_nodes.iter().next().copied();
        }
    }

    /// Toggle node selection
    pub fn toggle_selection(&mut self, node_id: NodeId) {
        if self.selected_nodes.contains(&node_id) {
            self.remove_from_selection(node_id);
        } else {
            self.add_to_selection(node_id);
        }
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.selected_nodes.clear();
        self.primary_node = None;
        self.selected_tracks.clear();
    }

    /// Select all nodes (Cmd/Ctrl + A)
    pub fn select_all(&mut self, node_ids: impl Iterator<Item = NodeId>) {
        self.selected_nodes.clear();
        for node_id in node_ids {
            self.selected_nodes.insert(node_id);
        }
        self.primary_node = self.selected_nodes.iter().next().copied();
    }

    /// Select nodes within a rectangle (drag selection)
    pub fn select_in_rect(&mut self, rect: egui::Rect, node_rects: &[(NodeId, egui::Rect)]) {
        for (node_id, node_rect) in node_rects {
            if rect.intersects(*node_rect) {
                self.selected_nodes.insert(*node_id);
            }
        }
        if self.primary_node.is_none() && !self.selected_nodes.is_empty() {
            self.primary_node = self.selected_nodes.iter().next().copied();
        }
    }

    /// Check if a node is selected
    pub fn is_selected(&self, node_id: &NodeId) -> bool {
        self.selected_nodes.contains(node_id)
    }

    /// Check if a node is the primary selection
    pub fn is_primary(&self, node_id: &NodeId) -> bool {
        self.primary_node.as_ref() == Some(node_id)
    }

    /// Get number of selected nodes
    pub fn count(&self) -> usize {
        self.selected_nodes.len()
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.selected_nodes.is_empty()
    }

    /// Get all selected node IDs as a vector
    pub fn selected_ids(&self) -> Vec<NodeId> {
        self.selected_nodes.iter().copied().collect()
    }

    /// Select track
    pub fn select_track(&mut self, track_id: TrackId) {
        self.selected_tracks.insert(track_id);
    }

    /// Check if track is selected
    pub fn is_track_selected(&self, track_id: &TrackId) -> bool {
        self.selected_tracks.contains(track_id)
    }
}

/// Rectangle selection state (for drag selection)
#[derive(Clone, Debug)]
pub struct RectSelection {
    pub start_pos: egui::Pos2,
    pub current_pos: egui::Pos2,
}

impl RectSelection {
    pub fn new(start_pos: egui::Pos2) -> Self {
        Self {
            start_pos,
            current_pos: start_pos,
        }
    }

    pub fn update(&mut self, current_pos: egui::Pos2) {
        self.current_pos = current_pos;
    }

    /// Get the selection rectangle
    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_two_pos(self.start_pos, self.current_pos)
    }
}
