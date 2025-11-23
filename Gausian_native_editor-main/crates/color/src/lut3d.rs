/// 3D LUT (Lookup Table) system
/// Phase 3: Advanced Color Management & LUTs

use anyhow::{Context, Result};
use std::path::Path;
use wgpu;

/// 3D LUT data structure
#[derive(Debug, Clone)]
pub struct Lut3D {
    /// LUT size (typically 17, 33, 65)
    pub size: u32,

    /// RGB triplet data (size^3 entries)
    pub data: Vec<[f32; 3]>,

    /// Input range (min, max)
    pub input_range: (f32, f32),

    /// LUT name/title
    pub name: String,

    /// File format
    pub format: LutFormat,
}

/// LUT file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LutFormat {
    Cube,   // Adobe .cube
    ThreeDL, // Autodesk .3dl
    Csp,    // Rising Sun .csp
}

impl Lut3D {
    /// Create new empty LUT
    pub fn new(size: u32, name: String) -> Self {
        let total_size = (size * size * size) as usize;
        let mut data = Vec::with_capacity(total_size);

        // Initialize with identity (no-op LUT)
        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    let rf = r as f32 / (size - 1) as f32;
                    let gf = g as f32 / (size - 1) as f32;
                    let bf = b as f32 / (size - 1) as f32;
                    data.push([rf, gf, bf]);
                }
            }
        }

        Self {
            size,
            data,
            input_range: (0.0, 1.0),
            name,
            format: LutFormat::Cube,
        }
    }

    /// Load LUT from .cube file
    pub fn from_cube_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read LUT file: {:?}", path))?;

        crate::parsers::cube::parse_cube(&content)
            .with_context(|| format!("Failed to parse .cube file: {:?}", path))
    }

    /// Load LUT from .3dl file
    pub fn from_3dl_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read LUT file: {:?}", path))?;

        crate::parsers::threedl::parse_3dl(&content)
            .with_context(|| format!("Failed to parse .3dl file: {:?}", path))
    }

    /// Auto-detect and load LUT file
    pub fn from_file(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "cube" => Self::from_cube_file(path),
            "3dl" => Self::from_3dl_file(path),
            _ => anyhow::bail!("Unsupported LUT format: {}", ext),
        }
    }

    /// Convert LUT to GPU texture (3D texture)
    pub fn to_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("LUT3D: {}", self.name)),
            size: wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: self.size,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Convert [f32; 3] to [f32; 4] (add alpha = 1.0)
        let rgba_data: Vec<f32> = self
            .data
            .iter()
            .flat_map(|[r, g, b]| [*r, *g, *b, 1.0])
            .collect();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&rgba_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.size * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(self.size),
            },
            wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: self.size,
            },
        );

        texture
    }

    /// Sample LUT at RGB coordinate (for CPU-side preview)
    pub fn sample(&self, r: f32, g: f32, b: f32) -> [f32; 3] {
        // Normalize input to LUT coordinates
        let r_norm = ((r - self.input_range.0) / (self.input_range.1 - self.input_range.0))
            .clamp(0.0, 1.0);
        let g_norm = ((g - self.input_range.0) / (self.input_range.1 - self.input_range.0))
            .clamp(0.0, 1.0);
        let b_norm = ((b - self.input_range.0) / (self.input_range.1 - self.input_range.0))
            .clamp(0.0, 1.0);

        // Map to LUT indices
        let r_idx = (r_norm * (self.size - 1) as f32) as usize;
        let g_idx = (g_norm * (self.size - 1) as f32) as usize;
        let b_idx = (b_norm * (self.size - 1) as f32) as usize;

        // Linear index
        let idx = b_idx * (self.size * self.size) as usize
            + g_idx * self.size as usize
            + r_idx;

        self.data[idx]
    }

    /// Get LUT info string
    pub fn info(&self) -> String {
        format!(
            "{} ({}³, {:?}, range: {:.2}-{:.2})",
            self.name, self.size, self.format, self.input_range.0, self.input_range.1
        )
    }
}
