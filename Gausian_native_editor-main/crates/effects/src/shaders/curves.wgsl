// RGB/Luma Curves Shader
// Applies curve transformations to each color channel independently
// Phase 2: Advanced Color Correction

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var master_curve: texture_1d<f32>;
@group(0) @binding(3) var red_curve: texture_1d<f32>;
@group(0) @binding(4) var green_curve: texture_1d<f32>;
@group(0) @binding(5) var blue_curve: texture_1d<f32>;

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

// Sample curve LUT
fn sample_curve(curve: texture_1d<f32>, value: f32) -> f32 {
    let clamped = clamp(value, 0.0, 1.0);
    // Sample at texel center
    let u = clamped * 255.0 / 256.0 + 0.5 / 256.0;
    return textureSampleLevel(curve, tex_sampler, u, 0.0).r;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample input color
    let color = textureSample(input_tex, tex_sampler, input.tex_coords);

    // Apply individual channel curves
    var r = sample_curve(red_curve, color.r);
    var g = sample_curve(green_curve, color.g);
    var b = sample_curve(blue_curve, color.b);

    // Apply master curve to all channels
    r = sample_curve(master_curve, r);
    g = sample_curve(master_curve, g);
    b = sample_curve(master_curve, b);

    // Preserve alpha
    return vec4<f32>(r, g, b, color.a);
}
