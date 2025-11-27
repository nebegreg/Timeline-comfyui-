/// Transform effect (position, scale, rotation)
/// Phase 2: Rich Effects & Transitions
use crate::{Effect, EffectCategory, EffectParameter, ParameterType};
use anyhow::Result;
use glam::{Mat3, Vec2};
use std::collections::HashMap;
use wgpu;
use wgpu::util::DeviceExt;

pub struct TransformEffect {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniform_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl TransformEffect {
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
            label: Some("Transform Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/transform.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Transform Texture Bind Group Layout"),
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
                label: Some("Transform Uniform Bind Group Layout"),
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
            label: Some("Transform Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Transform Pipeline"),
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

    fn build_transform_matrix(position: Vec2, scale: f32, rotation_degrees: f32) -> Mat3 {
        // Build 2D transformation matrix: Translation × Rotation × Scale
        let rotation_radians = rotation_degrees.to_radians();
        let cos_r = rotation_radians.cos();
        let sin_r = rotation_radians.sin();

        // Combine transformations (applied in reverse order: scale → rotate → translate)
        Mat3::from_cols(
            glam::Vec3::new(cos_r * scale, sin_r * scale, 0.0),
            glam::Vec3::new(-sin_r * scale, cos_r * scale, 0.0),
            glam::Vec3::new(position.x, position.y, 1.0),
        )
    }
}

impl Default for TransformEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for TransformEffect {
    fn name(&self) -> &str {
        "transform"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::Transform
    }

    fn parameters(&self) -> Vec<EffectParameter> {
        vec![
            EffectParameter {
                name: "position_x".to_string(),
                display_name: "Position X".to_string(),
                param_type: ParameterType::Slider,
                default: 0.0,
                min: -1920.0,
                max: 1920.0,
                description: "X position offset".to_string(),
            },
            EffectParameter {
                name: "position_y".to_string(),
                display_name: "Position Y".to_string(),
                param_type: ParameterType::Slider,
                default: 0.0,
                min: -1080.0,
                max: 1080.0,
                description: "Y position offset".to_string(),
            },
            EffectParameter {
                name: "scale".to_string(),
                display_name: "Scale".to_string(),
                param_type: ParameterType::Slider,
                default: 1.0,
                min: 0.01,
                max: 5.0,
                description: "Uniform scale".to_string(),
            },
            EffectParameter {
                name: "rotation".to_string(),
                display_name: "Rotation".to_string(),
                param_type: ParameterType::Angle,
                default: 0.0,
                min: 0.0,
                max: 360.0,
                description: "Rotation in degrees".to_string(),
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
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        self_mut.ensure_pipeline(device);

        let pipeline = self.pipeline.as_ref().unwrap();
        let bind_group_layout = self.bind_group_layout.as_ref().unwrap();
        let uniform_bind_group_layout = self.uniform_bind_group_layout.as_ref().unwrap();

        // Get parameters
        let pos_x = self.get_param(params, "position_x");
        let pos_y = self.get_param(params, "position_y");
        let scale = self.get_param(params, "scale");
        let rotation = self.get_param(params, "rotation");

        // Normalize position to [-0.5, 0.5] range
        // Assuming 1920x1080 reference resolution
        let position = Vec2::new(pos_x / 1920.0, pos_y / 1080.0);

        // Build transformation matrix
        let transform_matrix = Self::build_transform_matrix(position, scale, rotation);

        // Convert Mat3 to array for uniform buffer (column-major, 3x3 + padding)
        let matrix_data = transform_matrix.to_cols_array();
        let uniform_data = [
            matrix_data[0],
            matrix_data[1],
            matrix_data[2],
            0.0, // Column 1 + padding
            matrix_data[3],
            matrix_data[4],
            matrix_data[5],
            0.0, // Column 2 + padding
            matrix_data[6],
            matrix_data[7],
            matrix_data[8],
            0.0, // Column 3 + padding
        ];

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Uniforms"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Transform Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let input_view = input.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform Texture Bind Group"),
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
            label: Some("Transform Uniform Bind Group"),
            layout: uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Transform Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Transform Render Pass"),
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
