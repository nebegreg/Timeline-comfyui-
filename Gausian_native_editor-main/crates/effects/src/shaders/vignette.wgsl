// Vignette shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    intensity: f32,
    softness: f32,
    padding1: f32,
    padding2: f32,
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
    var color = textureSample(input_texture, tex_sampler, input.uv);

    // Calculate distance from center
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(input.uv - center);

    // Create vignette mask with smoothstep
    let radius = 0.7;  // Vignette starts at 70% from center
    let softness_factor = mix(0.1, 0.5, uniforms.softness);

    let vignette = smoothstep(
        radius,
        radius - softness_factor,
        dist
    );

    // Apply vignette (darken edges)
    let darkness = 1.0 - (uniforms.intensity * (1.0 - vignette));
    color = vec4<f32>(color.rgb * darkness, color.a);

    return color;
}
