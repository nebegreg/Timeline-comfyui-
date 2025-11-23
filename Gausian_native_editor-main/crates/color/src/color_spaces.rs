/// Color space utilities
/// Phase 3: Advanced Color Management & LUTs

use glam::Mat3;

/// Get color space conversion matrix
pub fn get_conversion_matrix(from: super::ColorSpace, to: super::ColorSpace) -> Mat3 {
    // TODO: Implement full matrix calculations
    // For now, return identity
    Mat3::IDENTITY
}

/// sRGB gamma correction (linearize)
pub fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB gamma correction (delinearize)
pub fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

/// Rec. 709 OETF (Opto-Electronic Transfer Function)
pub fn rec709_oetf(value: f32) -> f32 {
    if value < 0.018 {
        4.5 * value
    } else {
        1.099 * value.powf(0.45) - 0.099
    }
}

/// Rec. 709 EOTF (Electro-Optical Transfer Function)
pub fn rec709_eotf(value: f32) -> f32 {
    if value < 0.081 {
        value / 4.5
    } else {
        ((value + 0.099) / 1.099).powf(1.0 / 0.45)
    }
}

/// Rec. 2020 primaries (same as Rec. 709 transfer)
pub fn rec2020_to_linear(value: f32) -> f32 {
    rec709_eotf(value)
}

/// ACES reference rendering transform (simplified)
pub fn aces_rrt(color: [f32; 3]) -> [f32; 3] {
    // TODO: Implement full ACES RRT
    // For now, simple tonemap
    color.map(|c| c / (c + 1.0))
}
