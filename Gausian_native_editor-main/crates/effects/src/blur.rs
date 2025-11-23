/// Gaussian Blur effect
/// Phase 2: Rich Effects & Transitions

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};
use anyhow::Result;
use std::collections::HashMap;
use wgpu;

pub struct GaussianBlurEffect {
    // Two-pass separable blur (horizontal + vertical)
    horizontal_pipeline: Option<wgpu::RenderPipeline>,
    vertical_pipeline: Option<wgpu::RenderPipeline>,
    temp_texture: Option<wgpu::Texture>,
}

impl GaussianBlurEffect {
    pub fn new() -> Self {
        Self {
            horizontal_pipeline: None,
            vertical_pipeline: None,
            temp_texture: None,
        }
    }
}

impl Default for GaussianBlurEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for GaussianBlurEffect {
    fn name(&self) -> &str {
        "gaussian_blur"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::Blur
    }

    fn parameters(&self) -> &[EffectParameter] {
        &[EffectParameter {
            name: "radius".to_string(),
            display_name: "Blur Radius".to_string(),
            param_type: ParameterType::Slider,
            default: 5.0,
            min: 0.0,
            max: 100.0,
            description: "Blur radius in pixels".to_string(),
        }]
    }

    fn apply(
        &self,
        _input: &wgpu::Texture,
        _output: &wgpu::Texture,
        _params: &HashMap<String, f32>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Result<()> {
        // TODO: Implement two-pass separable blur
        // 1. Horizontal blur: input → temp_texture
        // 2. Vertical blur: temp_texture → output
        Ok(())
    }
}
