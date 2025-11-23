/// Color management and LUT system
/// Phase 3: Advanced Color Management & LUTs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use wgpu;

pub mod lut3d;
pub mod color_spaces;
pub mod parsers;
pub mod scopes;

pub use lut3d::Lut3D;
pub use color_spaces::*;
pub use scopes::{ScopeAnalyzer, ScopeData, ScopeType};

/// Color space identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    /// sRGB (standard)
    Srgb,

    /// Rec. 709 (HDTV)
    Rec709,

    /// Rec. 2020 (UHD)
    Rec2020,

    /// DCI-P3 (Cinema)
    DciP3,

    /// ACES 2065-1 (Archival)
    Aces2065,

    /// ACEScg (Working space)
    AcesCg,

    /// Linear RGB
    LinearRgb,
}

impl ColorSpace {
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            Self::Srgb => "sRGB",
            Self::Rec709 => "Rec. 709",
            Self::Rec2020 => "Rec. 2020",
            Self::DciP3 => "DCI-P3",
            Self::Aces2065 => "ACES 2065-1",
            Self::AcesCg => "ACEScg",
            Self::LinearRgb => "Linear RGB",
        }
    }

    /// Get all color spaces
    pub fn all() -> [Self; 7] {
        [
            Self::Srgb,
            Self::Rec709,
            Self::Rec2020,
            Self::DciP3,
            Self::Aces2065,
            Self::AcesCg,
            Self::LinearRgb,
        ]
    }
}

/// Color space transform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTransform {
    Rec709ToLinear,
    LinearToRec709,
    Rec2020ToAcesCg,
    AcesCgToRec2020,
    SrgbToLinear,
    LinearToSrgb,
    Rec709ToRec2020,
    Rec2020ToRec709,
}

impl ColorTransform {
    /// Get 3x3 transformation matrix
    pub fn matrix(&self) -> [[f32; 3]; 3] {
        match self {
            Self::Rec709ToLinear => {
                // BT.709 YUV to Linear RGB
                [
                    [1.164, 0.000, 1.596],
                    [1.164, -0.392, -0.813],
                    [1.164, 2.017, 0.000],
                ]
            }
            Self::LinearToRec709 => {
                // Inverse (simplified)
                [
                    [0.859, 0.000, -0.859],
                    [0.859, 0.275, 0.584],
                    [0.859, -1.734, 0.000],
                ]
            }
            Self::SrgbToLinear => {
                // Identity (handled by gamma)
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            }
            Self::LinearToSrgb => {
                // Identity (handled by gamma)
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            }
            Self::Rec709ToRec2020 => {
                // BT.709 to BT.2020 chromatic adaptation
                [
                    [0.627, 0.329, 0.043],
                    [0.069, 0.920, 0.011],
                    [0.016, 0.088, 0.896],
                ]
            }
            Self::Rec2020ToRec709 => {
                // Inverse
                [
                    [1.661, -0.588, -0.073],
                    [-0.125, 1.133, -0.008],
                    [-0.018, -0.100, 1.118],
                ]
            }
            _ => {
                // TODO: Implement other transforms
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            }
        }
    }
}
