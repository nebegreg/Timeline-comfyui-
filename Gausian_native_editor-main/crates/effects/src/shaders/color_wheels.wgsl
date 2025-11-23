// Color Wheels Shader
// Lift/Gamma/Gain color correction for Shadows/Midtones/Highlights
// Phase 2: Advanced Color Correction

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: ColorWheelsUniforms;

struct ColorWheelsUniforms {
    // Shadows (lift)
    shadows_hue: f32,
    shadows_saturation: f32,
    shadows_luminance: f32,
    _padding1: f32,

    // Midtones (gamma)
    midtones_hue: f32,
    midtones_saturation: f32,
    midtones_luminance: f32,
    _padding2: f32,

    // Highlights (gain)
    highlights_hue: f32,
    highlights_saturation: f32,
    highlights_luminance: f32,
    _padding3: f32,

    // Range thresholds
    shadow_max: f32,
    highlight_min: f32,
    blend_width: f32,
    intensity: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    // Full-screen triangle
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    output.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    output.tex_coords = vec2<f32>(x, y);

    return output;
}

// RGB to HSL conversion
fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
    let max_c = max(max(rgb.r, rgb.g), rgb.b);
    let min_c = min(min(rgb.r, rgb.g), rgb.b);
    let delta = max_c - min_c;

    var h: f32 = 0.0;
    var s: f32 = 0.0;
    let l: f32 = (max_c + min_c) / 2.0;

    if (delta > 0.0001) {
        if (l < 0.5) {
            s = delta / (max_c + min_c);
        } else {
            s = delta / (2.0 - max_c - min_c);
        }

        if (max_c == rgb.r) {
            h = ((rgb.g - rgb.b) / delta) + (select(0.0, 6.0, rgb.g < rgb.b));
        } else if (max_c == rgb.g) {
            h = ((rgb.b - rgb.r) / delta) + 2.0;
        } else {
            h = ((rgb.r - rgb.g) / delta) + 4.0;
        }
        h = h / 6.0;
    }

    return vec3<f32>(h, s, l);
}

// HSL to RGB conversion
fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let h = hsl.x;
    let s = hsl.y;
    let l = hsl.z;

    if (s < 0.0001) {
        return vec3<f32>(l, l, l);
    }

    var q: f32;
    if (l < 0.5) {
        q = l * (1.0 + s);
    } else {
        q = l + s - l * s;
    }
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0/3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0/3.0);

    return vec3<f32>(r, g, b);
}

fn hue_to_rgb(p: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if (t < 0.0) { t = t + 1.0; }
    if (t > 1.0) { t = t - 1.0; }
    if (t < 1.0/6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 1.0/2.0) { return q; }
    if (t < 2.0/3.0) { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
    return p;
}

// Smooth step blend
fn smooth_blend(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Calculate luminance weights
fn get_range_weights(luma: f32) -> vec3<f32> {
    let shadow_max = uniforms.shadow_max;
    let highlight_min = uniforms.highlight_min;
    let blend = uniforms.blend_width;

    var shadow_weight = 0.0;
    var midtone_weight = 0.0;
    var highlight_weight = 0.0;

    // Shadow weight: 1.0 below shadow_max, blend down
    shadow_weight = 1.0 - smooth_blend(shadow_max - blend, shadow_max + blend, luma);

    // Highlight weight: 1.0 above highlight_min, blend up
    highlight_weight = smooth_blend(highlight_min - blend, highlight_min + blend, luma);

    // Midtone weight: whatever's left
    midtone_weight = 1.0 - shadow_weight - highlight_weight;

    return vec3<f32>(shadow_weight, midtone_weight, highlight_weight);
}

// Apply color adjustment
fn apply_color_adjustment(rgb: vec3<f32>, hue_shift: f32, sat_mult: f32, lum_adjust: f32, mode: i32) -> vec3<f32> {
    var hsl = rgb_to_hsl(rgb);

    // Apply hue shift (in degrees, convert to 0-1 range)
    hsl.x = hsl.x + (hue_shift / 360.0);
    hsl.x = fract(hsl.x);  // Wrap around

    // Apply saturation
    hsl.y = clamp(hsl.y * sat_mult, 0.0, 1.0);

    // Apply luminance based on mode
    // mode 0 = lift (add), mode 1 = gamma (pow), mode 2 = gain (multiply)
    if (mode == 0) {
        // Lift: add offset
        hsl.z = clamp(hsl.z + lum_adjust, 0.0, 1.0);
    } else if (mode == 1) {
        // Gamma: power function
        hsl.z = pow(clamp(hsl.z, 0.0001, 1.0), lum_adjust);
    } else {
        // Gain: multiply
        hsl.z = clamp(hsl.z * lum_adjust, 0.0, 1.0);
    }

    return hsl_to_rgb(hsl);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_tex, tex_sampler, input.tex_coords);

    // Calculate luminance (Rec. 709)
    let luma = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    // Get range weights
    let weights = get_range_weights(luma);

    // Apply shadow adjustment (lift)
    var result = color.rgb;
    if (weights.x > 0.001) {
        let shadow_adjusted = apply_color_adjustment(
            result,
            uniforms.shadows_hue,
            uniforms.shadows_saturation,
            uniforms.shadows_luminance,
            0  // lift mode
        );
        result = mix(result, shadow_adjusted, weights.x);
    }

    // Apply midtone adjustment (gamma)
    if (weights.y > 0.001) {
        let midtone_adjusted = apply_color_adjustment(
            result,
            uniforms.midtones_hue,
            uniforms.midtones_saturation,
            uniforms.midtones_luminance,
            1  // gamma mode
        );
        result = mix(result, midtone_adjusted, weights.y);
    }

    // Apply highlight adjustment (gain)
    if (weights.z > 0.001) {
        let highlight_adjusted = apply_color_adjustment(
            result,
            uniforms.highlights_hue,
            uniforms.highlights_saturation,
            uniforms.highlights_luminance,
            2  // gain mode
        );
        result = mix(result, highlight_adjusted, weights.z);
    }

    // Blend with original based on intensity
    result = mix(color.rgb, result, uniforms.intensity);

    return vec4<f32>(result, color.a);
}
