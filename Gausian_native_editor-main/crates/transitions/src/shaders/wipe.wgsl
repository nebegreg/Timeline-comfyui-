// Wipe transition shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var from_texture: texture_2d<f32>;
@group(0) @binding(1) var to_texture: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct Uniforms {
    progress: f32,    // 0.0 to 1.0
    direction_x: f32, // Wipe direction X
    direction_y: f32, // Wipe direction Y
    feather: f32,     // Edge softness
}

@group(1) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;

    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let from_color = textureSample(from_texture, tex_sampler, input.uv);
    let to_color = textureSample(to_texture, tex_sampler, input.uv);

    // Calculate wipe position along direction
    let uv_centered = input.uv - 0.5;  // Center UV
    let position = dot(uv_centered, vec2<f32>(uniforms.direction_x, uniforms.direction_y));

    // Map to -0.5 to 0.5 range, then apply progress
    let threshold = uniforms.progress - 0.5;

    // Smoothstep for feathering
    let mix_factor = smoothstep(
        threshold - uniforms.feather,
        threshold + uniforms.feather,
        position
    );

    return mix(from_color, to_color, mix_factor);
}
