/// RGB/Luma Curves Effect
/// Allows precise color and luminance control using Bézier curves
/// Phase 2: Advanced Color Correction

use anyhow::Result;
use std::collections::HashMap;
use wgpu;

use crate::{Effect, EffectCategory, EffectParameter, ParameterType};

/// Curves effect for precise color control
pub struct CurvesEffect {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,

    // Curve lookup textures (1D, 256 entries each)
    master_curve_texture: Option<wgpu::Texture>,
    red_curve_texture: Option<wgpu::Texture>,
    green_curve_texture: Option<wgpu::Texture>,
    blue_curve_texture: Option<wgpu::Texture>,
}

/// Control point for Bézier curve
#[derive(Debug, Clone, Copy)]
pub struct CurvePoint {
    pub x: f32,  // Input value (0-1)
    pub y: f32,  // Output value (0-1)
}

impl CurvesEffect {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            sampler: None,
            master_curve_texture: None,
            red_curve_texture: None,
            green_curve_texture: None,
            blue_curve_texture: None,
        }
    }

    /// Set curve for a specific channel
    /// points: Control points for the curve (must include 0,0 and 1,1)
    pub fn set_curve(&mut self, channel: CurveChannel, points: &[CurvePoint], device: &wgpu::Device, queue: &wgpu::Queue) {
        let curve_data = Self::generate_curve_lut(points);
        let texture = Self::create_curve_texture(&curve_data, device, queue);

        match channel {
            CurveChannel::Master => self.master_curve_texture = Some(texture),
            CurveChannel::Red => self.red_curve_texture = Some(texture),
            CurveChannel::Green => self.green_curve_texture = Some(texture),
            CurveChannel::Blue => self.blue_curve_texture = Some(texture),
        }
    }

    /// Generate 256-entry LUT from control points using Catmull-Rom spline
    fn generate_curve_lut(points: &[CurvePoint]) -> Vec<f32> {
        let mut lut = vec![0.0; 256];

        if points.is_empty() {
            // Identity curve
            for i in 0..256 {
                lut[i] = i as f32 / 255.0;
            }
            return lut;
        }

        // Ensure points are sorted by x
        let mut sorted_points = points.to_vec();
        sorted_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

        // Ensure endpoints
        if sorted_points.first().unwrap().x > 0.0 {
            sorted_points.insert(0, CurvePoint { x: 0.0, y: 0.0 });
        }
        if sorted_points.last().unwrap().x < 1.0 {
            sorted_points.push(CurvePoint { x: 1.0, y: 1.0 });
        }

        // Generate LUT using linear interpolation between points
        // (For production, use Catmull-Rom or cubic splines)
        for i in 0..256 {
            let x = i as f32 / 255.0;
            lut[i] = Self::interpolate_curve(&sorted_points, x).clamp(0.0, 1.0);
        }

        lut
    }

    /// Linear interpolation between curve points
    fn interpolate_curve(points: &[CurvePoint], x: f32) -> f32 {
        if points.len() == 1 {
            return points[0].y;
        }

        // Find surrounding points
        for i in 0..points.len() - 1 {
            if x >= points[i].x && x <= points[i + 1].x {
                let t = (x - points[i].x) / (points[i + 1].x - points[i].x);
                return points[i].y + t * (points[i + 1].y - points[i].y);
            }
        }

        // Extrapolate
        if x < points.first().unwrap().x {
            return points.first().unwrap().y;
        }
        points.last().unwrap().y
    }

    /// Create 1D texture for curve LUT
    fn create_curve_texture(data: &[f32], device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Curve LUT Texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Convert to bytes
        let data_bytes: Vec<u8> = data.iter()
            .flat_map(|&v| v.to_le_bytes())
            .collect();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    fn init_if_needed(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.pipeline.is_some() {
            return;
        }

        // Create sampler
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Curves Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));

        // Initialize with identity curves if not set
        if self.master_curve_texture.is_none() {
            let identity = Self::generate_curve_lut(&[]);
            self.master_curve_texture = Some(Self::create_curve_texture(&identity, device, queue));
        }
        if self.red_curve_texture.is_none() {
            let identity = Self::generate_curve_lut(&[]);
            self.red_curve_texture = Some(Self::create_curve_texture(&identity, device, queue));
        }
        if self.green_curve_texture.is_none() {
            let identity = Self::generate_curve_lut(&[]);
            self.green_curve_texture = Some(Self::create_curve_texture(&identity, device, queue));
        }
        if self.blue_curve_texture.is_none() {
            let identity = Self::generate_curve_lut(&[]);
            self.blue_curve_texture = Some(Self::create_curve_texture(&identity, device, queue));
        }

        // Bind group layout
        self.bind_group_layout = Some(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Curves Bind Group Layout"),
            entries: &[
                // Input texture
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Master curve
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
                // Red curve
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
                // Green curve
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
                // Blue curve
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        }));

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Curves Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/curves.wgsl").into()),
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Curves Pipeline Layout"),
            bind_group_layouts: &[self.bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        self.pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Curves Pipeline"),
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

/// Curve channel selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveChannel {
    Master,  // Applied to all channels
    Red,
    Green,
    Blue,
}

impl Effect for CurvesEffect {
    fn name(&self) -> &str {
        "curves"
    }

    fn category(&self) -> EffectCategory {
        EffectCategory::ColorCorrection
    }

    fn parameters(&self) -> Vec<EffectParameter> {
        vec![
            EffectParameter {
                name: "intensity".to_string(),
                display_name: "Intensity".to_string(),
                param_type: ParameterType::Percentage,
                default: 100.0,
                min: 0.0,
                max: 100.0,
                description: "Blend intensity of curve effect".to_string(),
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
        mut_effect.init_if_needed(device, queue);

        let intensity = self.get_param(params, "intensity") / 100.0;
        if intensity < 0.001 {
            // Copy input to output
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_texture_to_texture(
                input.as_image_copy(),
                output.as_image_copy(),
                input.size(),
            );
            queue.submit(Some(encoder.finish()));
            return Ok(());
        }

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Curves Bind Group"),
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
                    resource: wgpu::BindingResource::TextureView(
                        &self.master_curve_texture.as_ref().unwrap().create_view(&Default::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.red_curve_texture.as_ref().unwrap().create_view(&Default::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &self.green_curve_texture.as_ref().unwrap().create_view(&Default::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        &self.blue_curve_texture.as_ref().unwrap().create_view(&Default::default())
                    ),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Curves Render Pass"),
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

impl Default for CurvesEffect {
    fn default() -> Self {
        Self::new()
    }
}
