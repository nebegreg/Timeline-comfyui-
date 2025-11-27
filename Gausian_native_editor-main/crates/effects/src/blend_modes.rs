/// Blend Modes Effect
/// Photoshop-style blend modes for compositing
/// Phase 2: Compositing
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu;

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};

/// Blend modes effect for layer compositing
pub struct BlendModesEffect {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    uniform_buffer: Option<wgpu::Buffer>,
    blend_layer_texture: Option<wgpu::Texture>,
}

/// Blend mode types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Overlay = 3,
    SoftLight = 4,
    HardLight = 5,
    ColorDodge = 6,
    ColorBurn = 7,
    Darken = 8,
    Lighten = 9,
    Difference = 10,
    Exclusion = 11,
    Add = 12,
    Subtract = 13,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct BlendUniforms {
    blend_mode: u32,
    opacity: f32,
    _padding1: f32,
    _padding2: f32,
}

impl BlendModesEffect {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            sampler: None,
            uniform_buffer: None,
            blend_layer_texture: None,
        }
    }

    /// Set the blend layer texture (layer to blend on top)
    pub fn set_blend_layer(&mut self, texture: wgpu::Texture) {
        self.blend_layer_texture = Some(texture);
    }

    fn init_if_needed(&mut self, device: &wgpu::Device) {
        if self.pipeline.is_some() {
            return;
        }

        // Create sampler
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blend Modes Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));

        // Uniform buffer
        self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Blend Modes Uniform Buffer"),
            size: std::mem::size_of::<BlendUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Bind group layout
        self.bind_group_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Blend Modes Bind Group Layout"),
                entries: &[
                    // Base layer (bottom)
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
                    // Blend layer (top)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        ));

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blend Modes Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blend_modes.wgsl").into()),
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend Modes Pipeline Layout"),
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        self.pipeline = Some(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Blend Modes Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            }),
        );
    }
}

impl Effect for BlendModesEffect {
    fn name(&self) -> &str {
        "blend_modes"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::Generate
    }

    fn parameters(&self) -> Vec<EffectParameter> {
        vec![
            EffectParameter {
                name: "blend_mode".to_string(),
                display_name: "Blend Mode".to_string(),
                param_type: ParameterType::Slider,
                default: BlendMode::Normal as u32 as f32,
                min: 0.0,
                max: 13.0,
                description: "Blend mode: 0=Normal, 1=Multiply, 2=Screen, 3=Overlay, 4=SoftLight, 5=HardLight, 6=ColorDodge, 7=ColorBurn, 8=Darken, 9=Lighten, 10=Difference, 11=Exclusion, 12=Add, 13=Subtract".to_string(),
            },
            EffectParameter {
                name: "opacity".to_string(),
                display_name: "Opacity".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: 0.0,
                max: 100.0,
                description: "Blend opacity".to_string(),
            },
        ]
    }

    fn apply(
        &mut self,
        input: &wgpu::Texture,
        output: &wgpu::Texture,
        params: &HashMap<String, f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        self.init_if_needed(device);

        // If no blend layer is set, just copy input to output
        if self.blend_layer_texture.is_none() {
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_texture_to_texture(
                input.as_image_copy(),
                output.as_image_copy(),
                input.size(),
            );
            queue.submit(Some(encoder.finish()));
            return Ok(());
        }

        // Build uniforms
        let uniforms = BlendUniforms {
            blend_mode: self.get_param(params, "blend_mode") as u32,
            opacity: self.get_param(params, "opacity") / 100.0,
            _padding1: 0.0,
            _padding2: 0.0,
        };

        // Update uniform buffer
        queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blend Modes Bind Group"),
            layout: self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &input.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self
                            .blend_layer_texture
                            .as_ref()
                            .unwrap()
                            .create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blend Modes Render Pass"),
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

impl Default for BlendModesEffect {
    fn default() -> Self {
        Self::new()
    }
}
