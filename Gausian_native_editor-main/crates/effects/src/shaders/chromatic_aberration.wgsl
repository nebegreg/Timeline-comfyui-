// Chromatic Aberration shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    strength: f32,
    center_x: f32,
    center_y: f32,
    padding: f32,
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

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let center = vec2<f32>(uniforms.center_x, uniforms.center_y);

    // Calculate direction from center
    let direction = input.uv - center;
    let distance = length(direction);
    let normalized_dir = normalize(direction);

    // Calculate offset based on distance from center
    let offset_strength = uniforms.strength / texture_size.x * distance;

    // Offset R, G, B channels radially
    // Red channel: shift outward
    let r_offset = normalized_dir * offset_strength;
    let r = textureSample(input_texture, tex_sampler, input.uv + r_offset).r;

    // Green channel: no shift (or minimal)
    let g = textureSample(input_texture, tex_sampler, input.uv).g;

    // Blue channel: shift inward
    let b_offset = normalized_dir * (-offset_strength);
    let b = textureSample(input_texture, tex_sampler, input.uv + b_offset).b;

    // Alpha from center sample
    let alpha = textureSample(input_texture, tex_sampler, input.uv).a;

    return vec4<f32>(r, g, b, alpha);
}
