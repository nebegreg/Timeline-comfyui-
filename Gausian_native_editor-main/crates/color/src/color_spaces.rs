/// Color space utilities
/// Phase 3: Advanced Color Management & LUTs

use glam::{Mat3, Vec3};

/// Get color space conversion matrix
pub fn get_conversion_matrix(from: super::ColorSpace, to: super::ColorSpace) -> Mat3 {
    use super::ColorSpace::*;

    match (from, to) {
        // Identity transforms
        (a, b) if a == b => Mat3::IDENTITY,

        // Rec.709 <-> Linear
        (Rec709, LinearRgb) => Mat3::IDENTITY, // Handled by transfer function
        (LinearRgb, Rec709) => Mat3::IDENTITY,

        // sRGB <-> Linear
        (Srgb, LinearRgb) => Mat3::IDENTITY, // Handled by transfer function
        (LinearRgb, Srgb) => Mat3::IDENTITY,

        // Rec.709 -> Rec.2020
        (Rec709, Rec2020) => Mat3::from_cols_array(&[
            0.627, 0.069, 0.016,
            0.329, 0.920, 0.088,
            0.043, 0.011, 0.896,
        ]),

        // Rec.2020 -> Rec.709
        (Rec2020, Rec709) => Mat3::from_cols_array(&[
            1.661, -0.125, -0.018,
            -0.588, 1.133, -0.100,
            -0.073, -0.008, 1.118,
        ]),

        // Rec.709 -> ACEScg
        (Rec709, AcesCg) => Mat3::from_cols_array(&[
            0.613, 0.070, 0.021,
            0.341, 0.918, 0.106,
            0.046, 0.012, 0.873,
        ]),

        // ACEScg -> Rec.709
        (AcesCg, Rec709) => Mat3::from_cols_array(&[
            1.705, -0.130, -0.024,
            -0.622, 1.141, -0.129,
            -0.083, -0.011, 1.153,
        ]),

        // ACEScg -> ACES 2065-1 (AP0)
        (AcesCg, Aces2065) => Mat3::from_cols_array(&[
            0.695, 0.140, 0.164,
            0.045, 0.860, 0.095,
            -0.001, 0.000, 1.001,
        ]),

        // ACES 2065-1 -> ACEScg
        (Aces2065, AcesCg) => Mat3::from_cols_array(&[
            1.451, -0.237, -0.214,
            -0.077, 1.176, -0.099,
            0.008, 0.001, 0.998,
        ]),

        // DCI-P3 -> Linear
        (DciP3, LinearRgb) => Mat3::from_cols_array(&[
            0.822, 0.033, 0.017,
            0.178, 0.967, 0.000,
            0.000, 0.000, 0.983,
        ]),

        // Linear -> DCI-P3
        (LinearRgb, DciP3) => Mat3::from_cols_array(&[
            1.225, -0.042, -0.020,
            -0.225, 1.042, 0.000,
            0.000, 0.000, 1.018,
        ]),

        // Default: return identity and handle in calling code
        _ => Mat3::IDENTITY,
    }
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

/// ACES Reference Rendering Transform (RRT)
/// Simplified version of ACES RRT for tone mapping
pub fn aces_rrt(color: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = color;
    let rgb = Vec3::new(r, g, b);

    // ACES tonemap curve (simplified Reinhard-like)
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;

    let tonemapped = rgb * (rgb * a + b) / (rgb * (rgb * c + d) + e);

    [
        tonemapped.x.clamp(0.0, 1.0),
        tonemapped.y.clamp(0.0, 1.0),
        tonemapped.z.clamp(0.0, 1.0),
    ]
}

/// ACES Output Device Transform (ODT) for sRGB/Rec.709
pub fn aces_odt_rec709(color: [f32; 3]) -> [f32; 3] {
    // Apply RRT first
    let rrt_color = aces_rrt(color);

    // Apply Rec.709 gamma
    rrt_color.map(|c| linear_to_srgb(c))
}

/// ACES Input Device Transform (IDT) for sRGB/Rec.709
pub fn aces_idt_rec709(color: [f32; 3]) -> [f32; 3] {
    // Linearize
    let linear = color.map(|c| srgb_to_linear(c));

    // Apply matrix transform to ACEScg
    let matrix = get_conversion_matrix(super::ColorSpace::Rec709, super::ColorSpace::AcesCg);
    let rgb = Vec3::from_array(linear);
    let transformed = matrix * rgb;

    transformed.to_array()
}

/// Full ACES workflow: IDT -> RRT -> ODT
pub fn aces_full_pipeline(input_color: [f32; 3], input_space: super::ColorSpace) -> [f32; 3] {
    // 1. Convert input to ACEScg
    let to_aces = get_conversion_matrix(input_space, super::ColorSpace::AcesCg);
    let rgb_in = Vec3::from_array(input_color);
    let aces_color = to_aces * rgb_in;

    // 2. Apply RRT
    let rrt_color = aces_rrt(aces_color.to_array());

    // 3. Apply ODT (convert back to display space)
    let from_aces = get_conversion_matrix(super::ColorSpace::AcesCg, input_space);
    let rgb_out = Vec3::from_array(rrt_color);
    let final_color = from_aces * rgb_out;

    final_color.to_array()
}

/// Apply color space transformation with transfer function
pub fn transform_color(
    color: [f32; 3],
    from: super::ColorSpace,
    to: super::ColorSpace,
) -> [f32; 3] {
    use super::ColorSpace::*;

    // Handle transfer functions (linearization)
    let linear_color = match from {
        Srgb => color.map(|c| srgb_to_linear(c)),
        Rec709 => color.map(|c| rec709_eotf(c)),
        Rec2020 => color.map(|c| rec2020_to_linear(c)),
        _ => color,
    };

    // Apply matrix transform
    let matrix = get_conversion_matrix(from, to);
    let rgb = Vec3::from_array(linear_color);
    let transformed = matrix * rgb;

    // Apply output transfer function
    let final_color = match to {
        Srgb => transformed.to_array().map(|c| linear_to_srgb(c)),
        Rec709 => transformed.to_array().map(|c| rec709_oetf(c)),
        Rec2020 => transformed.to_array().map(|c| rec709_oetf(c)),
        _ => transformed.to_array(),
    };

    final_color
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_roundtrip() {
        let original = 0.5f32;
        let linear = srgb_to_linear(original);
        let back = linear_to_srgb(linear);
        assert!((original - back).abs() < 0.001);
    }

    #[test]
    fn test_identity_transform() {
        let color = [0.5, 0.3, 0.7];
        let matrix = get_conversion_matrix(super::ColorSpace::Rec709, super::ColorSpace::Rec709);
        let rgb = Vec3::from_array(color);
        let result = matrix * rgb;
        let diff = (result - rgb).length();
        assert!(diff < 0.001);
    }

    #[test]
    fn test_aces_rrt() {
        let color = [1.5, 1.2, 0.8]; // HDR values
        let result = aces_rrt(color);
        // Should be tone-mapped to 0-1 range
        assert!(result[0] <= 1.0);
        assert!(result[1] <= 1.0);
        assert!(result[2] <= 1.0);
    }
}
