/// Timeline markers and regions system
/// Phase 1: Timeline Polish & UX Improvements

use crate::Frame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Marker ID
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct MarkerId(pub Uuid);

impl MarkerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MarkerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    /// Standard marker
    Standard,

    /// In point (edit range start)
    In,

    /// Out point (edit range end)
    Out,

    /// Chapter marker (for export)
    Chapter,

    /// Comment/note marker
    Comment,

    /// TODO marker
    Todo,
}

impl Default for MarkerType {
    fn default() -> Self {
        Self::Standard
    }
}

/// Timeline marker
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub id: MarkerId,
    pub frame: Frame,
    pub label: String,
    pub marker_type: MarkerType,

    /// Color in hex format (e.g., "#FF0000")
    #[serde(default = "default_marker_color")]
    pub color: String,

    /// Optional note/comment
    #[serde(default)]
    pub note: String,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: i64,
}

fn default_marker_color() -> String {
    "#4A9EFF".to_string() // Blue
}

impl Marker {
    pub fn new(frame: Frame, label: String) -> Self {
        Self {
            id: MarkerId::new(),
            frame,
            label,
            marker_type: MarkerType::Standard,
            color: default_marker_color(),
            note: String::new(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_type(mut self, marker_type: MarkerType) -> Self {
        self.marker_type = marker_type;
        self.color = match marker_type {
            MarkerType::In => "#00FF00".to_string(),      // Green
            MarkerType::Out => "#FF0000".to_string(),     // Red
            MarkerType::Chapter => "#FF00FF".to_string(), // Magenta
            MarkerType::Comment => "#FFFF00".to_string(), // Yellow
            MarkerType::Todo => "#FFA500".to_string(),    // Orange
            MarkerType::Standard => default_marker_color(),
        };
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = color;
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = note;
        self
    }
}

/// Timeline region (in/out range)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: MarkerId,
    pub start: Frame,
    pub end: Frame,
    pub label: String,

    /// Color in hex format
    #[serde(default = "default_region_color")]
    pub color: String,

    /// Optional note
    #[serde(default)]
    pub note: String,
}

fn default_region_color() -> String {
    "#4A9EFF80".to_string() // Blue with alpha
}

impl Region {
    pub fn new(start: Frame, end: Frame, label: String) -> Self {
        Self {
            id: MarkerId::new(),
            start,
            end,
            label,
            color: default_region_color(),
            note: String::new(),
        }
    }

    pub fn duration(&self) -> Frame {
        self.end - self.start
    }

    pub fn contains(&self, frame: Frame) -> bool {
        frame >= self.start && frame < self.end
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = color;
        self
    }
}

/// Marker collection for a timeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkerCollection {
    markers: HashMap<MarkerId, Marker>,
    regions: HashMap<MarkerId, Region>,
}

impl MarkerCollection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a marker
    pub fn add_marker(&mut self, marker: Marker) -> MarkerId {
        let id = marker.id;
        self.markers.insert(id, marker);
        id
    }

    /// Remove a marker
    pub fn remove_marker(&mut self, id: &MarkerId) -> Option<Marker> {
        self.markers.remove(id)
    }

    /// Get marker
    pub fn get_marker(&self, id: &MarkerId) -> Option<&Marker> {
        self.markers.get(id)
    }

    /// Get marker (mutable)
    pub fn get_marker_mut(&mut self, id: &MarkerId) -> Option<&mut Marker> {
        self.markers.get_mut(id)
    }

    /// Get all markers
    pub fn markers(&self) -> impl Iterator<Item = &Marker> {
        self.markers.values()
    }

    /// Get markers sorted by frame
    pub fn markers_sorted(&self) -> Vec<&Marker> {
        let mut markers: Vec<_> = self.markers.values().collect();
        markers.sort_by_key(|m| m.frame);
        markers
    }

    /// Find markers at frame
    pub fn markers_at(&self, frame: Frame, tolerance: Frame) -> Vec<&Marker> {
        self.markers
            .values()
            .filter(|m| (m.frame - frame).abs() <= tolerance)
            .collect()
    }

    /// Find nearest marker to frame
    pub fn nearest_marker(&self, frame: Frame) -> Option<&Marker> {
        self.markers
            .values()
            .min_by_key(|m| (m.frame - frame).abs())
    }

    /// Add a region
    pub fn add_region(&mut self, region: Region) -> MarkerId {
        let id = region.id;
        self.regions.insert(id, region);
        id
    }

    /// Remove a region
    pub fn remove_region(&mut self, id: &MarkerId) -> Option<Region> {
        self.regions.remove(id)
    }

    /// Get region
    pub fn get_region(&self, id: &MarkerId) -> Option<&Region> {
        self.regions.get(id)
    }

    /// Get all regions
    pub fn regions(&self) -> impl Iterator<Item = &Region> {
        self.regions.values()
    }

    /// Find regions containing frame
    pub fn regions_at(&self, frame: Frame) -> Vec<&Region> {
        self.regions
            .values()
            .filter(|r| r.contains(frame))
            .collect()
    }

    /// Clear all markers and regions
    pub fn clear(&mut self) {
        self.markers.clear();
        self.regions.clear();
    }

    /// Get In/Out markers for edit range
    pub fn get_in_out_range(&self) -> Option<(Frame, Frame)> {
        let in_marker = self
            .markers
            .values()
            .find(|m| m.marker_type == MarkerType::In)?;
        let out_marker = self
            .markers
            .values()
            .find(|m| m.marker_type == MarkerType::Out)?;
        Some((in_marker.frame, out_marker.frame))
    }

    /// Set In point
    pub fn set_in_point(&mut self, frame: Frame) -> MarkerId {
        // Remove existing In marker
        self.markers.retain(|_, m| m.marker_type != MarkerType::In);

        let marker = Marker::new(frame, "In".to_string())
            .with_type(MarkerType::In);
        self.add_marker(marker)
    }

    /// Set Out point
    pub fn set_out_point(&mut self, frame: Frame) -> MarkerId {
        // Remove existing Out marker
        self.markers.retain(|_, m| m.marker_type != MarkerType::Out);

        let marker = Marker::new(frame, "Out".to_string())
            .with_type(MarkerType::Out);
        self.add_marker(marker)
    }

    /// Clear In/Out points
    pub fn clear_in_out(&mut self) {
        self.markers.retain(|_, m| {
            m.marker_type != MarkerType::In && m.marker_type != MarkerType::Out
        });
    }
}
