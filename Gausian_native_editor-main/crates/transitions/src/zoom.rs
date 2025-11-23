/// Zoom Transition
/// Scale-based transition (zoom in/out)
/// Phase 2: Transitions System

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wgpu;

use crate::Transition;

/// Zoom transition effect
pub struct ZoomTransition {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    uniform_buffer: Option<wgpu::Buffer>,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ZoomUniforms {
    progress: f32,
    zoom_direction: f32,  // 0 = zoom out, 1 = zoom in
    feather: f32,
    _padding: f32,
}

impl ZoomTransition {
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
            label: Some("Zoom Transition Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));

        // Uniform buffer
        self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Zoom Transition Uniform Buffer"),
            size: std::mem::size_of::<ZoomUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Bind group layout
        self.bind_group_layout = Some(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Zoom Transition Bind Group Layout"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
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
        }));

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Zoom Transition Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/zoom.wgsl").into()),
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Zoom Transition Pipeline Layout"),
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        self.pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Zoom Transition Pipeline"),
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

impl Transition for ZoomTransition {
    fn name(&self) -> &str {
        "zoom"
    }

    fn render(
        &self,
        from_frame: &wgpu::Texture,
        to_frame: &wgpu::Texture,
        progress: f32,
        output: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        let mut transition = self;
        let mut_transition = unsafe {
            &mut *(transition as *const Self as *mut Self)
        };
        mut_transition.init_if_needed(device);

        // Build uniforms
        let uniforms = ZoomUniforms {
            progress,
            zoom_direction: 1.0,  // Default zoom in
            feather: 0.1,
            _padding: 0.0,
        };

        // Update uniform buffer
        queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Zoom Transition Bind Group"),
            layout: self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &from_frame.create_view(&Default::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &to_frame.create_view(&Default::default())
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
                label: Some("Zoom Transition Render Pass"),
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

impl Default for ZoomTransition {
    fn default() -> Self {
        Self::new()
    }
}
