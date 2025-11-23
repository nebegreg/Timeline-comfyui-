/// Autodesk .3dl LUT parser
/// Phase 3: Advanced Color Management & LUTs
///
/// .3dl format specifications:
/// - Text file with RGB triplets (one per line)
/// - Values typically in 0-1023 range (10-bit) or 0-4095 (12-bit)
/// - Common sizes: 17³, 32³, 33³, 64³, 65³
/// - No explicit size declaration - inferred from line count
/// - Comments start with #
/// - First non-comment line can be a title

use crate::lut3d::{Lut3D, LutFormat};
use anyhow::{Context, Result};

/// Parse .3dl file content
pub fn parse_3dl(content: &str) -> Result<Lut3D> {
    let mut title = String::from("Untitled");
    let mut data = Vec::new();
    let mut first_line = true;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // First line might be a title (if not numeric)
        if first_line {
            first_line = false;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 1 && parts[0].parse::<f32>().is_err() {
                title = line.to_string();
                continue;
            }
        }

        // Parse RGB data
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
            ) {
                data.push([r, g, b]);
            }
        }
    }

    // Infer LUT size from data count
    let data_len = data.len();
    let size = infer_lut_size(data_len)
        .with_context(|| format!("Cannot infer LUT size from {} entries", data_len))?;

    // Detect input range (10-bit: 0-1023, 12-bit: 0-4095, normalized: 0-1)
    let max_value = data
        .iter()
        .flat_map(|rgb| rgb.iter())
        .fold(0.0f32, |acc, &v| acc.max(v));

    let input_range = if max_value > 2.0 {
        // 10-bit or 12-bit data
        if max_value > 1100.0 {
            (0.0, 4095.0) // 12-bit
        } else {
            (0.0, 1023.0) // 10-bit
        }
    } else {
        (0.0, 1.0) // Normalized
    };

    // Normalize data to 0-1 range
    let normalized_data: Vec<[f32; 3]> = data
        .into_iter()
        .map(|[r, g, b]| {
            [
                r / input_range.1,
                g / input_range.1,
                b / input_range.1,
            ]
        })
        .collect();

    Ok(Lut3D {
        size,
        data: normalized_data,
        input_range: (0.0, 1.0), // Normalized
        name: title,
        format: LutFormat::ThreeDL,
    })
}

/// Infer LUT size from data entry count
fn infer_lut_size(count: usize) -> Option<u32> {
    // Common LUT sizes: 17³, 32³, 33³, 64³, 65³
    let common_sizes = [17, 32, 33, 64, 65];

    for size in common_sizes {
        if size * size * size == count {
            return Some(size as u32);
        }
    }

    // Try to find cube root
    let cube_root = (count as f64).cbrt().round() as usize;
    if cube_root * cube_root * cube_root == count {
        return Some(cube_root as u32);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_lut_size() {
        assert_eq!(infer_lut_size(4913), Some(17)); // 17³
        assert_eq!(infer_lut_size(32768), Some(32)); // 32³
        assert_eq!(infer_lut_size(35937), Some(33)); // 33³
        assert_eq!(infer_lut_size(262144), Some(64)); // 64³
        assert_eq!(infer_lut_size(274625), Some(65)); // 65³
        assert_eq!(infer_lut_size(8), Some(2)); // 2³
        assert_eq!(infer_lut_size(1000), None); // Not a cube
    }

    #[test]
    fn test_parse_simple_3dl() {
        // 2³ = 8 entries, 10-bit values
        let content = r#"
Test LUT
0 0 0
1023 0 0
0 1023 0
1023 1023 0
0 0 1023
1023 0 1023
0 1023 1023
1023 1023 1023
        "#;

        let lut = parse_3dl(content).unwrap();
        assert_eq!(lut.size, 2);
        assert_eq!(lut.name, "Test LUT");
        assert_eq!(lut.data.len(), 8);

        // Check normalization (1023 -> 1.0)
        assert!((lut.data[1][0] - 1.0).abs() < 0.01);
        assert!((lut.data[7][0] - 1.0).abs() < 0.01);
        assert!((lut.data[7][1] - 1.0).abs() < 0.01);
        assert!((lut.data[7][2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_normalized_3dl() {
        // Normalized 0-1 values
        let content = r#"
# Comment line
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
        "#;

        let lut = parse_3dl(content).unwrap();
        assert_eq!(lut.size, 2);
        assert_eq!(lut.data.len(), 8);
        assert!((lut.data[7][0] - 1.0).abs() < 0.01);
    }
}
