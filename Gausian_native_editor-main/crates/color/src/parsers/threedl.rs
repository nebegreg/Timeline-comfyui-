/// Autodesk .3dl LUT parser
/// Phase 3: Advanced Color Management & LUTs

use crate::lut3d::{Lut3D, LutFormat};
use anyhow::Result;

/// Parse .3dl file content
pub fn parse_3dl(content: &str) -> Result<Lut3D> {
    // TODO: Implement .3dl parser
    // Format is different from .cube
    anyhow::bail!(".3dl format not yet implemented")
}
