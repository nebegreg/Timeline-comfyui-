/// Gaussian Blur effect
/// Phase 2: Rich Effects & Transitions

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};
use anyhow::Result;
use std::collections::HashMap;
use wgpu;
use wgpu::util::DeviceExt;

pub struct GaussianBlurEffect {
    // Two-pass separable blur (horizontal + vertical)
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniform_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl GaussianBlurEffect {
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
            label: Some("Gaussian Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blur.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Blur Texture Bind Group Layout"),
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
                label: Some("Blur Uniform Bind Group Layout"),
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
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blur Pipeline"),
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

        let radius = self.get_param(params, "radius");

        // Create temporary texture for intermediate pass
        let temp_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Blur Temp Texture"),
            size: input.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blur Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Pass 1: Horizontal blur (input → temp)
        {
            let uniform_data = [radius, 1.0, 0.0, 0.0];  // direction = (1, 0) for horizontal
            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Blur Horizontal Uniforms"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let input_view = input.create_view(&wgpu::TextureViewDescriptor::default());
            let temp_view = temp_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blur Horizontal Texture Bind Group"),
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
                label: Some("Blur Horizontal Uniform Bind Group"),
                layout: uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Blur Horizontal Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur Horizontal Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &temp_view,
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
        }

        // Pass 2: Vertical blur (temp → output)
        {
            let uniform_data = [radius, 0.0, 1.0, 0.0];  // direction = (0, 1) for vertical
            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Blur Vertical Uniforms"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let temp_view = temp_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let output_view = output.create_view(&wgpu::TextureViewDescriptor::default());

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blur Vertical Texture Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&temp_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blur Vertical Uniform Bind Group"),
                layout: uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Blur Vertical Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur Vertical Pass"),
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
        }

        Ok(())
    }
}
