/// Sharpen/Unsharp Mask effect
/// Phase 2: Rich Effects & Transitions

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};
use anyhow::Result;
use std::collections::HashMap;
use wgpu;
use wgpu::util::DeviceExt;

pub struct SharpenEffect {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniform_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl SharpenEffect {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            uniform_bind_group_layout: None,
        }
    }

    fn ensure_pipeline(&mut self, device: &wgpu::Device) {
        if self.pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sharpen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sharpen.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sharpen Texture Bind Group Layout"),
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
                ],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sharpen Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sharpen Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sharpen Pipeline"),
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
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);
        self.bind_group_layout = Some(texture_bind_group_layout);
        self.uniform_bind_group_layout = Some(uniform_bind_group_layout);
    }
}

impl Default for SharpenEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for SharpenEffect {
    fn name(&self) -> &str {
        "sharpen"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::Sharpen
    }

    fn parameters(&self) -> &[EffectParameter] {
        &[
            EffectParameter {
                name: "strength".to_string(),
                display_name: "Strength".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.0,
                max: 5.0,
                description: "Sharpening strength".to_string(),
            },
            EffectParameter {
                name: "radius".to_string(),
                display_name: "Radius".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.5,
                max: 3.0,
                description: "Sharpening radius in pixels".to_string(),
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
        let mut self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        self_mut.ensure_pipeline(device);

        let pipeline = self.pipeline.as_ref().unwrap();
        let bind_group_layout = self.bind_group_layout.as_ref().unwrap();
        let uniform_bind_group_layout = self.uniform_bind_group_layout.as_ref().unwrap();

        let strength = self.get_param(params, "strength");
        let radius = self.get_param(params, "radius");

        let uniform_data = [strength, radius, 0.0, 0.0];
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sharpen Uniforms"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sharpen Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let input_view = input.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sharpen Texture Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sharpen Uniform Bind Group"),
            layout: uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Sharpen Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sharpen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_bind_group(1, &uniform_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
