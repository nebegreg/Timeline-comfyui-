/// Color Wheels Effect
/// Separate color correction for Shadows, Midtones, and Highlights
/// Phase 2: Advanced Color Correction

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu;

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};

/// Color wheels effect for advanced color grading
pub struct ColorWheelsEffect {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    uniform_buffer: Option<wgpu::Buffer>,
}

/// Color wheel uniform data
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ColorWheelsUniforms {
    // Shadows (lift)
    shadows_hue: f32,
    shadows_saturation: f32,
    shadows_luminance: f32,
    _padding1: f32,

    // Midtones (gamma)
    midtones_hue: f32,
    midtones_saturation: f32,
    midtones_luminance: f32,
    _padding2: f32,

    // Highlights (gain)
    highlights_hue: f32,
    highlights_saturation: f32,
    highlights_luminance: f32,
    _padding3: f32,

    // Range thresholds
    shadow_max: f32,      // Luminance below this = shadows
    highlight_min: f32,   // Luminance above this = highlights
    blend_width: f32,     // Smooth transition width
    intensity: f32,       // Overall effect strength
}

impl ColorWheelsEffect {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            sampler: None,
            uniform_buffer: None,
        }
    }

    fn init_if_needed(&mut self, device: &wgpu::Device) {
        if self.pipeline.is_some() {
            return;
        }

        // Create sampler
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Color Wheels Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));

        // Uniform buffer
        self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Color Wheels Uniform Buffer"),
            size: std::mem::size_of::<ColorWheelsUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Bind group layout
        self.bind_group_layout = Some(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Wheels Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }));

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Color Wheels Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/color_wheels.wgsl").into()),
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Wheels Pipeline Layout"),
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        self.pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Color Wheels Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        }));
    }
}

impl Effect for ColorWheelsEffect {
    fn name(&self) -> &str {
        "color_wheels"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::ColorCorrection
    }

    fn parameters(&self) -> Vec<EffectParameter> {
        vec![
            // Shadows
            EffectParameter {
                name: "shadows_hue".to_string(),
                display_name: "Shadows Hue".to_string(),
                param_type: ParameterType::Angle,
                default: 0.0,
                min: -180.0,
                max: 180.0,
                description: "Hue shift for shadow regions".to_string(),
            },
            EffectParameter {
                name: "shadows_saturation".to_string(),
                display_name: "Shadows Saturation".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.0,
                max: 2.0,
                description: "Saturation multiplier for shadows".to_string(),
            },
            EffectParameter {
                name: "shadows_luminance".to_string(),
                display_name: "Shadows Lift".to_string(),
                param_type: ParameterType::Slider,
                default: 0.0,
                min: -0.5,
                max: 0.5,
                description: "Luminance offset for shadows (lift)".to_string(),
            },

            // Midtones
            EffectParameter {
                name: "midtones_hue".to_string(),
                display_name: "Midtones Hue".to_string(),
                param_type: ParameterType::Angle,
                default: 0.0,
                min: -180.0,
                max: 180.0,
                description: "Hue shift for midtone regions".to_string(),
            },
            EffectParameter {
                name: "midtones_saturation".to_string(),
                display_name: "Midtones Saturation".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.0,
                max: 2.0,
                description: "Saturation multiplier for midtones".to_string(),
            },
            EffectParameter {
                name: "midtones_luminance".to_string(),
                display_name: "Midtones Gamma".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.1,
                max: 3.0,
                description: "Gamma correction for midtones".to_string(),
            },

            // Highlights
            EffectParameter {
                name: "highlights_hue".to_string(),
                display_name: "Highlights Hue".to_string(),
                param_type: ParameterType::Angle,
                default: 0.0,
                min: -180.0,
                max: 180.0,
                description: "Hue shift for highlight regions".to_string(),
            },
            EffectParameter {
                name: "highlights_saturation".to_string(),
                display_name: "Highlights Saturation".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.0,
                max: 2.0,
                description: "Saturation multiplier for highlights".to_string(),
            },
            EffectParameter {
                name: "highlights_luminance".to_string(),
                display_name: "Highlights Gain".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.5,
                max: 2.0,
                description: "Luminance multiplier for highlights (gain)".to_string(),
            },

            // Range controls
            EffectParameter {
                name: "shadow_max".to_string(),
                display_name: "Shadow Range".to_string(),
                param_type: ParameterType::Slider,
                default: 0.3,
                min: 0.0,
                max: 1.0,
                description: "Upper luminance threshold for shadows".to_string(),
            },
            EffectParameter {
                name: "highlight_min".to_string(),
                display_name: "Highlight Range".to_string(),
                param_type: ParameterType::Slider,
                default: 0.7,
                min: 0.0,
                max: 1.0,
                description: "Lower luminance threshold for highlights".to_string(),
            },
            EffectParameter {
                name: "blend_width".to_string(),
                display_name: "Blend Width".to_string(),
                param_type: ParameterType::Slider,
                default: 0.1,
                min: 0.0,
                max: 0.5,
                description: "Smoothness of transitions between ranges".to_string(),
            },
            EffectParameter {
                name: "intensity".to_string(),
                display_name: "Intensity".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: 0.0,
                max: 100.0,
                description: "Overall effect strength".to_string(),
            },
        ]
    }

    fn apply(
        &self,
        input: &wgpu::Texture,
        output: &wgpu::Texture,
        params: &HashMap<String, f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        let mut effect = self;
        let mut_effect = unsafe {
            &mut *(effect as *const Self as *mut Self)
        };
        mut_effect.init_if_needed(device);

        // Build uniforms
        let uniforms = ColorWheelsUniforms {
            shadows_hue: self.get_param(params, "shadows_hue"),
            shadows_saturation: self.get_param(params, "shadows_saturation"),
            shadows_luminance: self.get_param(params, "shadows_luminance"),
            _padding1: 0.0,

            midtones_hue: self.get_param(params, "midtones_hue"),
            midtones_saturation: self.get_param(params, "midtones_saturation"),
            midtones_luminance: self.get_param(params, "midtones_luminance"),
            _padding2: 0.0,

            highlights_hue: self.get_param(params, "highlights_hue"),
            highlights_saturation: self.get_param(params, "highlights_saturation"),
            highlights_luminance: self.get_param(params, "highlights_luminance"),
            _padding3: 0.0,

            shadow_max: self.get_param(params, "shadow_max"),
            highlight_min: self.get_param(params, "highlight_min"),
            blend_width: self.get_param(params, "blend_width"),
            intensity: self.get_param(params, "intensity") / 100.0,
        };

        // Update uniform buffer
        queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Wheels Bind Group"),
            layout: self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &input.create_view(&Default::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Color Wheels Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output.create_view(&Default::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

impl Default for ColorWheelsEffect {
    fn default() -> Self {
        Self::new()
    }
}
