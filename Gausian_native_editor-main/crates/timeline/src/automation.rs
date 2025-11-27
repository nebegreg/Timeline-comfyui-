/// Animation & Keyframing System - Automation Engine
///
/// This module implements the interpolation engine for automation lanes,
/// calculating animated values between keyframes with various easing functions.
///
/// # Features
/// - Linear interpolation
/// - Bézier curve interpolation (cubic)
/// - Step interpolation (instant jumps)
/// - Hold interpolation (maintain value)
/// - Standard easing functions (ease-in, ease-out, ease-in-out)
/// - Custom easing with tangent control
use crate::{
    graph::{AutomationInterpolation, AutomationKeyframe, AutomationLane, KeyframeEasing},
    Frame,
};

/// Result type for automation operations
pub type Result<T> = std::result::Result<T, AutomationError>;

/// Errors that can occur during automation evaluation
#[derive(Debug, thiserror::Error)]
pub enum AutomationError {
    #[error("No keyframes in automation lane")]
    NoKeyframes,

    #[error("Invalid keyframe range: {0}")]
    InvalidRange(String),

    #[error("Invalid Bézier parameters: {0}")]
    InvalidBezier(String),
}

/// Main automation evaluation engine
pub struct AutomationEngine;

impl AutomationEngine {
    /// Evaluate the value of an automation lane at a specific frame
    ///
    /// # Arguments
    /// * `lane` - The automation lane to evaluate
    /// * `frame` - The frame at which to evaluate the automation
    ///
    /// # Returns
    /// The interpolated value at the given frame, or an error if evaluation fails
    ///
    /// # Example
    /// ```ignore
    /// let value = AutomationEngine::evaluate(&lane, 100)?;
    /// ```
    pub fn evaluate(lane: &AutomationLane, frame: Frame) -> Result<f64> {
        if lane.keyframes.is_empty() {
            return Err(AutomationError::NoKeyframes);
        }

        // Sort keyframes by frame (should be pre-sorted, but ensure it)
        let mut keyframes = lane.keyframes.clone();
        keyframes.sort_by_key(|kf| kf.frame);

        // Find the keyframe pair surrounding the current frame
        let result = Self::find_keyframe_pair(&keyframes, frame);

        match result {
            KeyframePair::Before(kf) => {
                // Before first keyframe - use first value
                Ok(kf.value)
            }
            KeyframePair::After(kf) => {
                // After last keyframe - use last value
                Ok(kf.value)
            }
            KeyframePair::Exact(kf) => {
                // Exactly on a keyframe
                Ok(kf.value)
            }
            KeyframePair::Between(kf1, kf2) => {
                // Between two keyframes - interpolate
                Self::interpolate(kf1, kf2, frame, &lane.interpolation)
            }
        }
    }

    /// Find the pair of keyframes surrounding a given frame
    fn find_keyframe_pair<'a>(
        keyframes: &'a [AutomationKeyframe],
        frame: Frame,
    ) -> KeyframePair<'a> {
        // Handle edge cases
        if keyframes.is_empty() {
            panic!("Should not happen - already checked");
        }

        if keyframes.len() == 1 {
            return KeyframePair::Exact(&keyframes[0]);
        }

        // Check if before first or after last
        if frame < keyframes[0].frame {
            return KeyframePair::Before(&keyframes[0]);
        }

        if frame >= keyframes[keyframes.len() - 1].frame {
            return KeyframePair::After(&keyframes[keyframes.len() - 1]);
        }

        // Find the surrounding pair
        for i in 0..keyframes.len() - 1 {
            let kf1 = &keyframes[i];
            let kf2 = &keyframes[i + 1];

            if frame == kf1.frame {
                return KeyframePair::Exact(kf1);
            }

            if frame == kf2.frame {
                return KeyframePair::Exact(kf2);
            }

            if frame > kf1.frame && frame < kf2.frame {
                return KeyframePair::Between(kf1, kf2);
            }
        }

        // Fallback (should not happen with correct logic)
        KeyframePair::After(&keyframes[keyframes.len() - 1])
    }

    /// Interpolate between two keyframes
    fn interpolate(
        kf1: &AutomationKeyframe,
        kf2: &AutomationKeyframe,
        frame: Frame,
        interpolation: &AutomationInterpolation,
    ) -> Result<f64> {
        // Calculate normalized time (0.0 to 1.0)
        let frame_diff = (kf2.frame - kf1.frame) as f64;
        if frame_diff <= 0.0 {
            return Err(AutomationError::InvalidRange(
                "Keyframes must have different frames".into(),
            ));
        }

        let t = ((frame - kf1.frame) as f64) / frame_diff;

        // Apply easing to get eased time
        let eased_t = Self::apply_easing(t, &kf2.easing)?;

        // Apply interpolation method
        let value = match interpolation {
            AutomationInterpolation::Step => {
                // Step: instant jump at kf2
                if t < 1.0 {
                    kf1.value
                } else {
                    kf2.value
                }
            }
            AutomationInterpolation::Linear => {
                // Linear interpolation with easing
                Self::lerp(kf1.value, kf2.value, eased_t)
            }
            AutomationInterpolation::Bezier => {
                // Cubic Bézier interpolation
                Self::bezier_interpolate(kf1.value, kf2.value, eased_t)
            }
        };

        Ok(value)
    }

    /// Apply easing function to normalized time
    fn apply_easing(t: f64, easing: &KeyframeEasing) -> Result<f64> {
        let t = t.clamp(0.0, 1.0);

        Ok(match easing {
            KeyframeEasing::Linear => t,

            KeyframeEasing::EaseIn => {
                // Cubic ease-in: t^3
                t * t * t
            }

            KeyframeEasing::EaseOut => {
                // Cubic ease-out: 1 - (1-t)^3
                let inv = 1.0 - t;
                1.0 - (inv * inv * inv)
            }

            KeyframeEasing::EaseInOut => {
                // Cubic ease-in-out (smooth S-curve)
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let inv = -2.0 * t + 2.0;
                    1.0 - (inv * inv * inv) / 2.0
                }
            }

            KeyframeEasing::Custom {
                in_tangent,
                out_tangent,
            } => {
                // Custom Bézier curve with control points
                Self::cubic_bezier_1d(t, *in_tangent, *out_tangent)
            }
        })
    }

    /// Linear interpolation
    #[inline]
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    /// Cubic Bézier interpolation (simplified for value interpolation)
    fn bezier_interpolate(start: f64, end: f64, t: f64) -> f64 {
        // Standard cubic Bézier with control points at 1/3 and 2/3
        let cp1 = start + (end - start) / 3.0;
        let cp2 = start + (end - start) * 2.0 / 3.0;

        // Cubic Bézier formula: B(t) = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        mt3 * start + 3.0 * mt2 * t * cp1 + 3.0 * mt * t2 * cp2 + t3 * end
    }

    /// Cubic Bézier curve for 1D easing (0 to 1)
    /// Uses tangent values to define control points
    fn cubic_bezier_1d(t: f64, in_tangent: f32, out_tangent: f32) -> f64 {
        let t = t.clamp(0.0, 1.0);

        // Control points based on tangents (clamped to reasonable values)
        let cp1 = (in_tangent as f64).clamp(0.0, 1.0);
        let cp2 = (out_tangent as f64).clamp(0.0, 1.0);

        // Cubic Bézier with P0=(0,0), P1=(cp1, 0.33), P2=(cp2, 0.66), P3=(1,1)
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        // Y value only (X is always t)
        3.0 * mt2 * t * (0.33 + cp1 * 0.34) + 3.0 * mt * t2 * (0.66 + cp2 * 0.34) + t3
    }

    /// Batch evaluate multiple frames (useful for preview/rendering)
    pub fn evaluate_range(
        lane: &AutomationLane,
        start_frame: Frame,
        end_frame: Frame,
    ) -> Result<Vec<(Frame, f64)>> {
        let mut results = Vec::new();

        for frame in start_frame..=end_frame {
            let value = Self::evaluate(lane, frame)?;
            results.push((frame, value));
        }

        Ok(results)
    }
}

/// Represents the relationship between a frame and keyframes
enum KeyframePair<'a> {
    /// Frame is before the first keyframe
    Before(&'a AutomationKeyframe),
    /// Frame is after the last keyframe
    After(&'a AutomationKeyframe),
    /// Frame is exactly on a keyframe
    Exact(&'a AutomationKeyframe),
    /// Frame is between two keyframes
    Between(&'a AutomationKeyframe, &'a AutomationKeyframe),
}

/// Helper functions for working with automation lanes
impl AutomationLane {
    /// Add a keyframe to the lane, maintaining sorted order
    pub fn add_keyframe(&mut self, keyframe: AutomationKeyframe) {
        // Remove any existing keyframe at the same frame
        self.keyframes.retain(|kf| kf.frame != keyframe.frame);

        // Add new keyframe
        self.keyframes.push(keyframe);

        // Sort by frame
        self.keyframes.sort_by_key(|kf| kf.frame);
    }

    /// Remove a keyframe at a specific frame
    pub fn remove_keyframe(&mut self, frame: Frame) -> Option<AutomationKeyframe> {
        if let Some(idx) = self.keyframes.iter().position(|kf| kf.frame == frame) {
            Some(self.keyframes.remove(idx))
        } else {
            None
        }
    }

    /// Get a keyframe at a specific frame
    pub fn get_keyframe(&self, frame: Frame) -> Option<&AutomationKeyframe> {
        self.keyframes.iter().find(|kf| kf.frame == frame)
    }

    /// Get a mutable keyframe at a specific frame
    pub fn get_keyframe_mut(&mut self, frame: Frame) -> Option<&mut AutomationKeyframe> {
        self.keyframes.iter_mut().find(|kf| kf.frame == frame)
    }

    /// Find the nearest keyframe to a given frame
    pub fn find_nearest_keyframe(&self, frame: Frame) -> Option<&AutomationKeyframe> {
        if self.keyframes.is_empty() {
            return None;
        }

        self.keyframes
            .iter()
            .min_by_key(|kf| (kf.frame - frame).abs())
    }

    /// Get all keyframes in a frame range
    pub fn keyframes_in_range(&self, start: Frame, end: Frame) -> Vec<&AutomationKeyframe> {
        self.keyframes
            .iter()
            .filter(|kf| kf.frame >= start && kf.frame <= end)
            .collect()
    }

    /// Clear all keyframes
    pub fn clear_keyframes(&mut self) {
        self.keyframes.clear();
    }

    /// Get the frame range covered by keyframes
    pub fn keyframe_range(&self) -> Option<(Frame, Frame)> {
        if self.keyframes.is_empty() {
            return None;
        }

        let min_frame = self.keyframes.iter().map(|kf| kf.frame).min()?;
        let max_frame = self.keyframes.iter().map(|kf| kf.frame).max()?;

        Some((min_frame, max_frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{AutomationTarget, LaneId, NodeId};

    fn create_test_lane() -> AutomationLane {
        AutomationLane {
            id: LaneId::new(),
            target: AutomationTarget {
                node: NodeId::new(),
                parameter: "opacity".to_string(),
            },
            interpolation: AutomationInterpolation::Linear,
            keyframes: vec![
                AutomationKeyframe {
                    frame: 0,
                    value: 0.0,
                    easing: KeyframeEasing::Linear,
                },
                AutomationKeyframe {
                    frame: 100,
                    value: 1.0,
                    easing: KeyframeEasing::Linear,
                },
            ],
        }
    }

    #[test]
    fn test_linear_interpolation() {
        let lane = create_test_lane();

        // Test exact keyframes
        assert_eq!(AutomationEngine::evaluate(&lane, 0).unwrap(), 0.0);
        assert_eq!(AutomationEngine::evaluate(&lane, 100).unwrap(), 1.0);

        // Test midpoint
        let mid_value = AutomationEngine::evaluate(&lane, 50).unwrap();
        assert!((mid_value - 0.5).abs() < 0.001);

        // Test quarter point
        let quarter_value = AutomationEngine::evaluate(&lane, 25).unwrap();
        assert!((quarter_value - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_before_first_keyframe() {
        let lane = create_test_lane();
        let value = AutomationEngine::evaluate(&lane, -10).unwrap();
        assert_eq!(value, 0.0); // Should hold first value
    }

    #[test]
    fn test_after_last_keyframe() {
        let lane = create_test_lane();
        let value = AutomationEngine::evaluate(&lane, 200).unwrap();
        assert_eq!(value, 1.0); // Should hold last value
    }

    #[test]
    fn test_ease_in() {
        let mut lane = create_test_lane();
        lane.keyframes[1].easing = KeyframeEasing::EaseIn;

        let mid_value = AutomationEngine::evaluate(&lane, 50).unwrap();
        // With ease-in, midpoint should be less than 0.5
        assert!(mid_value < 0.5);
    }

    #[test]
    fn test_ease_out() {
        let mut lane = create_test_lane();
        lane.keyframes[1].easing = KeyframeEasing::EaseOut;

        let mid_value = AutomationEngine::evaluate(&lane, 50).unwrap();
        // With ease-out, midpoint should be greater than 0.5
        assert!(mid_value > 0.5);
    }

    #[test]
    fn test_step_interpolation() {
        let mut lane = create_test_lane();
        lane.interpolation = AutomationInterpolation::Step;

        // Before step, should be first value
        let before_value = AutomationEngine::evaluate(&lane, 50).unwrap();
        assert_eq!(before_value, 0.0);

        // At step, should be second value
        let at_value = AutomationEngine::evaluate(&lane, 100).unwrap();
        assert_eq!(at_value, 1.0);
    }

    #[test]
    fn test_add_keyframe() {
        let mut lane = create_test_lane();

        // Add keyframe in the middle
        lane.add_keyframe(AutomationKeyframe {
            frame: 50,
            value: 0.75,
            easing: KeyframeEasing::Linear,
        });

        assert_eq!(lane.keyframes.len(), 3);

        // Verify it's at the right position
        let value = AutomationEngine::evaluate(&lane, 50).unwrap();
        assert_eq!(value, 0.75);
    }

    #[test]
    fn test_remove_keyframe() {
        let mut lane = create_test_lane();
        let removed = lane.remove_keyframe(0);

        assert!(removed.is_some());
        assert_eq!(lane.keyframes.len(), 1);
    }

    #[test]
    fn test_find_nearest_keyframe() {
        let lane = create_test_lane();

        let nearest = lane.find_nearest_keyframe(60).unwrap();
        assert_eq!(nearest.frame, 100);

        let nearest2 = lane.find_nearest_keyframe(40).unwrap();
        assert_eq!(nearest2.frame, 0);
    }

    #[test]
    fn test_keyframe_range() {
        let lane = create_test_lane();
        let (min, max) = lane.keyframe_range().unwrap();

        assert_eq!(min, 0);
        assert_eq!(max, 100);
    }

    #[test]
    fn test_evaluate_range() {
        let lane = create_test_lane();
        let results = AutomationEngine::evaluate_range(&lane, 0, 10).unwrap();

        assert_eq!(results.len(), 11); // 0 to 10 inclusive
        assert_eq!(results[0].1, 0.0);
        assert!((results[10].1 - 0.1).abs() < 0.001);
    }
}
