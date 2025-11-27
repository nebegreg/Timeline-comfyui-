/// Corner Pin Effect
/// 4-point perspective transformation for geometric distortion
/// Phase 2: Geometric Effects
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wgpu;

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};

/// Corner pin effect for perspective transforms
pub struct CornerPinEffect {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    uniform_buffer: Option<wgpu::Buffer>,
}

/// Perspective transform uniform data
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CornerPinUniforms {
    // Top-left corner
    tl_x: f32,
    tl_y: f32,
    // Top-right corner
    tr_x: f32,
    tr_y: f32,
    // Bottom-left corner
    bl_x: f32,
    bl_y: f32,
    // Bottom-right corner
    br_x: f32,
    br_y: f32,

    // Perspective matrix (3x3, stored as 4x4 for alignment)
    transform_matrix: [[f32; 4]; 4],
}

impl CornerPinEffect {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            sampler: None,
            uniform_buffer: None,
        }
    }

    /// Calculate perspective transformation matrix from 4 corner points
    /// Uses homography estimation via Direct Linear Transform (DLT)
    fn calculate_perspective_matrix(
        tl: [f32; 2],
        tr: [f32; 2],
        bl: [f32; 2],
        br: [f32; 2],
    ) -> [[f32; 4]; 4] {
        // Source corners (unit square normalized coordinates)
        let src = [
            [0.0, 0.0], // Top-left
            [1.0, 0.0], // Top-right
            [0.0, 1.0], // Bottom-left
            [1.0, 1.0], // Bottom-right
        ];

        // Destination corners
        let dst = [tl, tr, bl, br];

        // Compute homography matrix H using Direct Linear Transform (DLT)
        // We solve: dst = H * src for the 3x3 homography matrix H
        // This is a simplified version using bilinear approximation

        // Calculate deltas for bilinear interpolation
        let dx1 = tr[0] - br[0];
        let dx2 = bl[0] - br[0];
        let dx3 = tl[0] - tr[0] - bl[0] + br[0];

        let dy1 = tr[1] - br[1];
        let dy2 = bl[1] - br[1];
        let dy3 = tl[1] - tr[1] - bl[1] + br[1];

        // Compute perspective coefficients
        let det = dx1 * dy2 - dx2 * dy1;
        let g = if det.abs() > 1e-10 {
            (dx3 * dy2 - dx2 * dy3) / det
        } else {
            0.0
        };
        let h = if det.abs() > 1e-10 {
            (dx1 * dy3 - dx3 * dy1) / det
        } else {
            0.0
        };

        // Build 3x3 homography matrix (stored in 4x4 for GPU alignment)
        let mut matrix = [[0.0f32; 4]; 4];

        // First row
        matrix[0][0] = tr[0] - tl[0] + g * tr[0];
        matrix[0][1] = bl[0] - tl[0] + h * bl[0];
        matrix[0][2] = tl[0];
        matrix[0][3] = 0.0;

        // Second row
        matrix[1][0] = tr[1] - tl[1] + g * tr[1];
        matrix[1][1] = bl[1] - tl[1] + h * bl[1];
        matrix[1][2] = tl[1];
        matrix[1][3] = 0.0;

        // Third row (perspective divide)
        matrix[2][0] = g;
        matrix[2][1] = h;
        matrix[2][2] = 1.0;
        matrix[2][3] = 0.0;

        // Fourth row (not used, set to identity)
        matrix[3][0] = 0.0;
        matrix[3][1] = 0.0;
        matrix[3][2] = 0.0;
        matrix[3][3] = 1.0;

        matrix
    }

    fn init_if_needed(&mut self, device: &wgpu::Device) {
        if self.pipeline.is_some() {
            return;
        }

        // Create sampler
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Corner Pin Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));

        // Uniform buffer
        self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Corner Pin Uniform Buffer"),
            size: std::mem::size_of::<CornerPinUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Bind group layout
        self.bind_group_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Corner Pin Bind Group Layout"),
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
            },
        ));

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Corner Pin Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/corner_pin.wgsl").into()),
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Corner Pin Pipeline Layout"),
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        self.pipeline = Some(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Corner Pin Pipeline"),
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
            }),
        );
    }
}

impl Effect for CornerPinEffect {
    fn name(&self) -> &str {
        "corner_pin"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::Distort
    }

    fn parameters(&self) -> Vec<EffectParameter> {
        vec![
            // Top-left corner
            EffectParameter {
                name: "tl_x".to_string(),
                display_name: "Top-Left X".to_string(),
                param_type: ParameterType::Percentage,
                default: 0.0,
                min: -100.0,
                max: 200.0,
                description: "Top-left corner X position".to_string(),
            },
            EffectParameter {
                name: "tl_y".to_string(),
                display_name: "Top-Left Y".to_string(),
                param_type: ParameterType::Percentage,
                default: 0.0,
                min: -100.0,
                max: 200.0,
                description: "Top-left corner Y position".to_string(),
            },
            // Top-right corner
            EffectParameter {
                name: "tr_x".to_string(),
                display_name: "Top-Right X".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: -100.0,
                max: 200.0,
                description: "Top-right corner X position".to_string(),
            },
            EffectParameter {
                name: "tr_y".to_string(),
                display_name: "Top-Right Y".to_string(),
                param_type: ParameterType::Percentage,
                default: 0.0,
                min: -100.0,
                max: 200.0,
                description: "Top-right corner Y position".to_string(),
            },
            // Bottom-left corner
            EffectParameter {
                name: "bl_x".to_string(),
                display_name: "Bottom-Left X".to_string(),
                param_type: ParameterType::Percentage,
                default: 0.0,
                min: -100.0,
                max: 200.0,
                description: "Bottom-left corner X position".to_string(),
            },
            EffectParameter {
                name: "bl_y".to_string(),
                display_name: "Bottom-Left Y".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: -100.0,
                max: 200.0,
                description: "Bottom-left corner Y position".to_string(),
            },
            // Bottom-right corner
            EffectParameter {
                name: "br_x".to_string(),
                display_name: "Bottom-Right X".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: -100.0,
                max: 200.0,
                description: "Bottom-right corner X position".to_string(),
            },
            EffectParameter {
                name: "br_y".to_string(),
                display_name: "Bottom-Right Y".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: -100.0,
                max: 200.0,
                description: "Bottom-right corner Y position".to_string(),
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

        // Get corner positions (convert from percentage to normalized 0-1)
        let tl = [
            self.get_param(params, "tl_x") / 100.0,
            self.get_param(params, "tl_y") / 100.0,
        ];
        let tr = [
            self.get_param(params, "tr_x") / 100.0,
            self.get_param(params, "tr_y") / 100.0,
        ];
        let bl = [
            self.get_param(params, "bl_x") / 100.0,
            self.get_param(params, "bl_y") / 100.0,
        ];
        let br = [
            self.get_param(params, "br_x") / 100.0,
            self.get_param(params, "br_y") / 100.0,
        ];

        // Calculate perspective matrix
        let matrix = Self::calculate_perspective_matrix(tl, tr, bl, br);

        // Build uniforms
        let uniforms = CornerPinUniforms {
            tl_x: tl[0],
            tl_y: tl[1],
            tr_x: tr[0],
            tr_y: tr[1],
            bl_x: bl[0],
            bl_y: bl[1],
            br_x: br[0],
            br_y: br[1],
            transform_matrix: matrix,
        };

        // Update uniform buffer
        queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Corner Pin Bind Group"),
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
                label: Some("Corner Pin Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output.create_view(&Default::default()),
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

            render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

impl Default for CornerPinEffect {
    fn default() -> Self {
        Self::new()
    }
}
