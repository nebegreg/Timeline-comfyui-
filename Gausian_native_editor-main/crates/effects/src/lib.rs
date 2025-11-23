/// GPU-accelerated effects system for video editing
/// Phase 2: Rich Effects & Transitions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wgpu;

/// Effect trait - all effects must implement this
pub trait Effect: Send + Sync {
    /// Effect name (unique identifier)
    fn name(&self) -> &str;

    /// Effect category
    fn category(&self) -> EffectCategory;

    /// Effect parameters
    fn parameters(&self) -> &[EffectParameter];

    /// Apply effect to texture
    fn apply(
        &self,
        input: &wgpu::Texture,
        output: &wgpu::Texture,
        params: &HashMap<String, f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()>;

    /// Get parameter value (with default if not set)
    fn get_param(&self, params: &HashMap<String, f32>, name: &str) -> f32 {
        params.get(name).copied().unwrap_or_else(|| {
            self.parameters()
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.default)
                .unwrap_or(0.0)
        })
    }
}

/// Effect category for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectCategory {
    ColorCorrection,
    Stylize,
    Blur,
    Sharpen,
    Distort,
    Generate,
    Keying,
    Transform,
}

/// Effect parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectParameter {
    pub name: String,
    pub display_name: String,
    pub param_type: ParameterType,
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub description: String,
}

/// Parameter type for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Slider,
    Angle,       // 0-360 degrees
    Percentage,  // 0-100
    Color,       // RGB color
    Boolean,     // 0 or 1
}

/// Effect instance applied to a clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInstance {
    pub effect_id: String,
    pub enabled: bool,
    pub parameters: HashMap<String, f32>,

    /// Keyframes for parameter animation (frame → value)
    #[serde(default)]
    pub keyframes: HashMap<String, Vec<Keyframe>>,
}

/// Keyframe for parameter animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub frame: i64,
    pub value: f32,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Hold,
}

/// Effect manager - registry of available effects
pub struct EffectManager {
    effects: HashMap<String, Box<dyn Effect>>,
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    /// Register an effect
    pub fn register(&mut self, effect: Box<dyn Effect>) {
        let name = effect.name().to_string();
        self.effects.insert(name, effect);
    }

    /// Get effect by name
    pub fn get(&self, name: &str) -> Option<&Box<dyn Effect>> {
        self.effects.get(name)
    }

    /// List all effect names
    pub fn list_effects(&self) -> Vec<&str> {
        self.effects.keys().map(|s| s.as_str()).collect()
    }

    /// Get effects by category
    pub fn effects_in_category(&self, category: EffectCategory) -> Vec<&Box<dyn Effect>> {
        self.effects
            .values()
            .filter(|e| e.category() == category)
            .collect()
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

// Modules for specific effects (to be implemented)
pub mod brightness_contrast;
pub mod blur;
pub mod transform;

// Re-exports
pub use brightness_contrast::BrightnessContrastEffect;
pub use blur::GaussianBlurEffect;
pub use transform::TransformEffect;
