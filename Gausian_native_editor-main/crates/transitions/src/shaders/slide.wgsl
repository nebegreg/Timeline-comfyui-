// Slide transition shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var from_texture: texture_2d<f32>;
@group(0) @binding(1) var to_texture: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct Uniforms {
    progress: f32,   // 0.0 to 1.0
    offset_x: f32,   // Direction vector X
    offset_y: f32,   // Direction vector Y
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

    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;

    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate UV offsets for sliding motion
    let offset = vec2<f32>(uniforms.offset_x, uniforms.offset_y) * uniforms.progress;

    // "From" texture slides out in the direction
    let from_uv = input.uv + offset;

    // "To" texture slides in from the opposite direction
    let to_uv = input.uv + offset - vec2<f32>(uniforms.offset_x, uniforms.offset_y);

    // Determine which texture is visible at this UV coordinate
    // If from_uv is in bounds, show from texture
    // Otherwise show to texture

    let in_from_bounds = from_uv.x >= 0.0 && from_uv.x <= 1.0 &&
                         from_uv.y >= 0.0 && from_uv.y <= 1.0;

    let in_to_bounds = to_uv.x >= 0.0 && to_uv.x <= 1.0 &&
                       to_uv.y >= 0.0 && to_uv.y <= 1.0;

    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    if (in_from_bounds) {
        result = textureSample(from_texture, tex_sampler, from_uv);
    } else if (in_to_bounds) {
        result = textureSample(to_texture, tex_sampler, to_uv);
    }

    return result;
}
