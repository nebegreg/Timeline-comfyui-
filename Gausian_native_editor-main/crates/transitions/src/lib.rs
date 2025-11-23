/// GPU-accelerated transitions for video editing
/// Phase 2: Rich Effects & Transitions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use wgpu;

/// Transition trait - all transitions must implement this
pub trait Transition: Send + Sync {
    /// Transition name (unique identifier)
    fn name(&self) -> &str;

    /// Transition category
    fn category(&self) -> TransitionCategory;

    /// Default duration in frames
    fn default_duration(&self) -> i64 {
        30  // 30 frames at 30fps = 1 second
    }

    /// Render transition between two frames
    ///
    /// # Arguments
    /// * `from_frame` - Outgoing frame texture
    /// * `to_frame` - Incoming frame texture
    /// * `progress` - Transition progress (0.0 = from_frame, 1.0 = to_frame)
    /// * `output` - Output texture
    /// * `device` - WGPU device
    /// * `queue` - WGPU queue
    fn render(
        &mut self,
        from_frame: &wgpu::Texture,
        to_frame: &wgpu::Texture,
        progress: f32,
        output: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()>;
}

/// Transition category for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionCategory {
    Dissolve,
    Wipe,
    Slide,
    Zoom,
    Rotate,
    Shape,
    Custom,
}

/// Transition instance in timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionInstance {
    pub transition_id: String,
    pub start_frame: i64,
    pub duration_frames: i64,
    pub reversed: bool,
}

impl TransitionInstance {
    pub fn new(transition_id: String, start_frame: i64, duration_frames: i64) -> Self {
        Self {
            transition_id,
            start_frame,
            duration_frames,
            reversed: false,
        }
    }

    /// Calculate progress at a given frame (0.0 to 1.0)
    pub fn progress_at_frame(&self, frame: i64) -> f32 {
        if frame < self.start_frame {
            return 0.0;
        }
        if frame >= self.start_frame + self.duration_frames {
            return 1.0;
        }

        let t = (frame - self.start_frame) as f32 / self.duration_frames as f32;

        if self.reversed {
            1.0 - t
        } else {
            t
        }
    }

    /// Check if frame is within transition range
    pub fn contains_frame(&self, frame: i64) -> bool {
        frame >= self.start_frame && frame < self.start_frame + self.duration_frames
    }
}

/// Easing functions for smooth transitions
pub mod easing {
    pub fn linear(t: f32) -> f32 {
        t
    }

    pub fn ease_in_quad(t: f32) -> f32 {
        t * t
    }

    pub fn ease_out_quad(t: f32) -> f32 {
        t * (2.0 - t)
    }

    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    pub fn ease_in_cubic(t: f32) -> f32 {
        t * t * t
    }

    pub fn ease_out_cubic(t: f32) -> f32 {
        let t = t - 1.0;
        t * t * t + 1.0
    }

    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            let t = 2.0 * t - 2.0;
            1.0 + t * t * t / 2.0
        }
    }
}

// Modules for specific transitions
pub mod dissolve;
pub mod wipe;
pub mod slide;
pub mod zoom;
pub mod spin;

// Re-exports
pub use dissolve::DissolveTransition;
pub use wipe::WipeTransition;
pub use slide::SlideTransition;
pub use zoom::ZoomTransition;
pub use spin::SpinTransition;
