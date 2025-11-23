// Sharpen/Unsharp Mask shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    strength: f32,
    radius: f32,
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
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let texel_size = 1.0 / texture_size;

    // Original pixel
    let original = textureSample(input_texture, tex_sampler, input.uv);

    // Laplacian kernel for edge detection (unsharp mask)
    // Sample 8 neighbors + center
    let radius = uniforms.radius * texel_size;

    var laplacian = vec3<f32>(0.0);

    // Center weight
    laplacian += original.rgb * 8.0;

    // 8-connected neighbors
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(-radius.x, 0.0)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(radius.x, 0.0)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(0.0, -radius.y)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(0.0, radius.y)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(-radius.x, -radius.y)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(radius.x, -radius.y)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(-radius.x, radius.y)).rgb;
    laplacian -= textureSample(input_texture, tex_sampler, input.uv + vec2<f32>(radius.x, radius.y)).rgb;

    // Apply sharpening: original + strength * edge_detection
    var sharpened = original.rgb + laplacian * uniforms.strength;

    // Clamp to valid range
    sharpened = clamp(sharpened, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(sharpened, original.a);
}
