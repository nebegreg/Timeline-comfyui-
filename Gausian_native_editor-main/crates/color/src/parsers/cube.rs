/// Adobe .cube LUT parser
/// Phase 3: Advanced Color Management & LUTs

use crate::lut3d::{Lut3D, LutFormat};
use anyhow::{Context, Result};

/// Parse .cube file content
pub fn parse_cube(content: &str) -> Result<Lut3D> {
    let mut size: Option<u32> = None;
    let mut title = String::from("Untitled");
    let mut domain_min = [0.0f32; 3];
    let mut domain_max = [1.0f32; 3];
    let mut data = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse TITLE
        if line.starts_with("TITLE") {
            if let Some(t) = line.strip_prefix("TITLE").map(|s| s.trim()) {
                title = t.trim_matches('"').to_string();
            }
            continue;
        }

        // Parse LUT_3D_SIZE
        if line.starts_with("LUT_3D_SIZE") {
            if let Some(s) = line.strip_prefix("LUT_3D_SIZE").map(|s| s.trim()) {
                size = s.parse().ok();
            }
            continue;
        }

        // Parse DOMAIN_MIN
        if line.starts_with("DOMAIN_MIN") {
            if let Some(vals) = line.strip_prefix("DOMAIN_MIN").map(|s| s.trim()) {
                let parts: Vec<&str> = vals.split_whitespace().collect();
                if parts.len() == 3 {
                    domain_min[0] = parts[0].parse().unwrap_or(0.0);
                    domain_min[1] = parts[1].parse().unwrap_or(0.0);
                    domain_min[2] = parts[2].parse().unwrap_or(0.0);
                }
            }
            continue;
        }

        // Parse DOMAIN_MAX
        if line.starts_with("DOMAIN_MAX") {
            if let Some(vals) = line.strip_prefix("DOMAIN_MAX").map(|s| s.trim()) {
                let parts: Vec<&str> = vals.split_whitespace().collect();
                if parts.len() == 3 {
                    domain_max[0] = parts[0].parse().unwrap_or(1.0);
                    domain_max[1] = parts[1].parse().unwrap_or(1.0);
                    domain_max[2] = parts[2].parse().unwrap_or(1.0);
                }
            }
            continue;
        }

        // Parse RGB data
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
            ) {
                data.push([r, g, b]);
            }
        }
    }

    let size = size.context("Missing LUT_3D_SIZE in .cube file")?;
    let expected_entries = (size * size * size) as usize;

    if data.len() != expected_entries {
        anyhow::bail!(
            "LUT data size mismatch: expected {}, got {}",
            expected_entries,
            data.len()
        );
    }

    // Use max domain value for input range (assume symmetric)
    let input_max = domain_max[0].max(domain_max[1]).max(domain_max[2]);

    Ok(Lut3D {
        size,
        data,
        input_range: (0.0, input_max),
        name: title,
        format: LutFormat::Cube,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_cube() {
        let content = r#"
TITLE "Test LUT"
LUT_3D_SIZE 2

0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
        "#;

        let lut = parse_cube(content).unwrap();
        assert_eq!(lut.size, 2);
        assert_eq!(lut.name, "Test LUT");
        assert_eq!(lut.data.len(), 8);
    }
}
