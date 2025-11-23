// Saturation/Hue shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    saturation: f32,  // 0-2 range (1.0 = normal)
    hue_shift: f32,   // Radians to rotate hue
}

@group(1) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    // Full-screen quad
    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;

    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return output;
}

// RGB to HSL conversion
fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
    let max_val = max(max(rgb.r, rgb.g), rgb.b);
    let min_val = min(min(rgb.r, rgb.g), rgb.b);
    let delta = max_val - min_val;

    var h: f32 = 0.0;
    var s: f32 = 0.0;
    let l = (max_val + min_val) * 0.5;

    if (delta > 0.00001) {
        s = delta / (1.0 - abs(2.0 * l - 1.0));

        if (max_val == rgb.r) {
            h = ((rgb.g - rgb.b) / delta) % 6.0;
        } else if (max_val == rgb.g) {
            h = ((rgb.b - rgb.r) / delta) + 2.0;
        } else {
            h = ((rgb.r - rgb.g) / delta) + 4.0;
        }

        h = h * 60.0;  // Convert to degrees
        if (h < 0.0) {
            h += 360.0;
        }
    }

    return vec3<f32>(h, s, l);
}

// HSL to RGB conversion
fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let h = hsl.x;
    let s = hsl.y;
    let l = hsl.z;

    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let x = c * (1.0 - abs(((h / 60.0) % 2.0) - 1.0));
    let m = l - c * 0.5;

    var rgb: vec3<f32>;
    if (h < 60.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 120.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 180.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 240.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 300.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + m;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(input_texture, tex_sampler, input.uv);

    // Convert to HSL
    var hsl = rgb_to_hsl(color.rgb);

    // Apply saturation
    hsl.y *= uniforms.saturation;
    hsl.y = clamp(hsl.y, 0.0, 1.0);

    // Apply hue shift (convert radians to degrees)
    let hue_shift_degrees = uniforms.hue_shift * 57.29578;  // 180/PI
    hsl.x += hue_shift_degrees;

    // Wrap hue to 0-360 range
    hsl.x = hsl.x % 360.0;
    if (hsl.x < 0.0) {
        hsl.x += 360.0;
    }

    // Convert back to RGB
    let rgb = hsl_to_rgb(hsl);

    return vec4<f32>(rgb, color.a);
}
