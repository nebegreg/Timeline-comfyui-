// Brightness/Contrast shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    brightness: f32,
    contrast: f32,
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

    // Apply brightness (additive)
    color = vec4<f32>(
        color.rgb + vec3<f32>(uniforms.brightness),
        color.a
    );

    // Apply contrast (multiply around 0.5 gray)
    let gray = vec3<f32>(0.5);
    color = vec4<f32>(
        gray + (color.rgb - gray) * uniforms.contrast,
        color.a
    );

    return color;
}
