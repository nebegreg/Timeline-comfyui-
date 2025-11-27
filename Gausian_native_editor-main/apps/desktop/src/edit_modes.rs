/// Professional edit modes for timeline
/// Phase 1: Timeline Polish & UX Improvements
use serde::{Deserialize, Serialize};

/// Edit mode determines how timeline editing operations behave
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditMode {
    /// Normal editing - clips move independently
    Normal,

    /// Ripple Edit - moving/trimming affects all following clips on the same track
    /// When you trim the end of a clip, all clips after it shift
    Ripple,

    /// Roll Edit - adjusts the edit point between two adjacent clips
    /// One clip's out point and the next clip's in point move together
    Roll,

    /// Slide Edit - moves clip content without changing its position on timeline
    /// Changes the in/out points but keeps timeline position fixed
    Slide,

    /// Slip Edit - changes what portion of media is visible
    /// Moves the in/out points of the source media together
    Slip,
}

impl Default for EditMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl EditMode {
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            Self::Normal => "Normal",
            Self::Ripple => "Ripple",
            Self::Roll => "Roll",
            Self::Slide => "Slide",
            Self::Slip => "Slip",
        }
    }

    /// Get keyboard shortcut hint
    pub fn shortcut(&self) -> &str {
        match self {
            Self::Normal => "N",
            Self::Ripple => "R",
            Self::Roll => "T",
            Self::Slide => "S",
            Self::Slip => "Y",
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            Self::Normal => "Move and trim clips independently",
            Self::Ripple => "Shift all following clips when editing",
            Self::Roll => "Adjust edit point between two clips",
            Self::Slide => "Change media timing without moving clip",
            Self::Slip => "Change visible portion of media",
        }
    }

    /// Cycle to next edit mode
    pub fn next(&self) -> Self {
        match self {
            Self::Normal => Self::Ripple,
            Self::Ripple => Self::Roll,
            Self::Roll => Self::Slide,
            Self::Slide => Self::Slip,
            Self::Slip => Self::Normal,
        }
    }

    /// Get all modes for UI display
    pub fn all() -> [Self; 5] {
        [
            Self::Normal,
            Self::Ripple,
            Self::Roll,
            Self::Slide,
            Self::Slip,
        ]
    }
}

/// Snapping configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapSettings {
    /// Enable snapping
    pub enabled: bool,

    /// Snap to playhead
    pub snap_to_playhead: bool,

    /// Snap to clip edges
    pub snap_to_clips: bool,

    /// Snap to markers
    pub snap_to_markers: bool,

    /// Snap to second boundaries
    pub snap_to_seconds: bool,

    /// Snap tolerance in pixels
    pub snap_tolerance: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_to_playhead: true,
            snap_to_clips: true,
            snap_to_markers: true,
            snap_to_seconds: true,
            snap_tolerance: 5.0,
        }
    }
}

impl SnapSettings {
    /// Toggle snapping on/off
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}
