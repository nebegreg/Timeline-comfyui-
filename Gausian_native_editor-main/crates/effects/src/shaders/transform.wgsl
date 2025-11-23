// Transform shader
// Phase 2: Rich Effects & Transitions
//
// Supports: Position, Scale, Rotation, Anchor point

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    transform_matrix: mat3x3<f32>,  // 2D transformation matrix
    padding: f32,                   // Alignment padding
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
    // Transform UV coordinates
    // Convert UV from [0,1] to [-0.5, 0.5] for center-based transform
    var uv = input.uv - 0.5;

    // Apply transformation matrix
    let transformed = uniforms.transform_matrix * vec3<f32>(uv.x, uv.y, 1.0);

    // Convert back to [0,1] range
    let final_uv = transformed.xy + 0.5;

    // Sample with transformed coordinates
    // Return transparent if outside texture bounds
    if (final_uv.x < 0.0 || final_uv.x > 1.0 || final_uv.y < 0.0 || final_uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return textureSample(input_texture, tex_sampler, final_uv);
}
