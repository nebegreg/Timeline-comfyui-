// 3D LUT Application Shader
// Phase 3: Advanced Color Management & LUTs
//
// Applies a 3D LUT to an input texture for color grading

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var lut_texture: texture_3d<f32>;
@group(0) @binding(3) var lut_sampler: sampler;

struct Uniforms {
    lut_size: f32,
    intensity: f32,    // Blend factor (0.0 = original, 1.0 = full LUT)
    _padding1: f32,
    _padding2: f32,
}

@group(1) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen quad
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Sample input color
    let input_color = textureSample(input_texture, input_sampler, uv);

    // Clamp to 0-1 range for LUT lookup
    let rgb = clamp(input_color.rgb, vec3<f32>(0.0), vec3<f32>(1.0));

    // Calculate 3D texture coordinates
    // We need to offset by half a texel to properly sample the LUT
    let scale = (uniforms.lut_size - 1.0) / uniforms.lut_size;
    let offset = 0.5 / uniforms.lut_size;
    let lut_coord = rgb * scale + vec3<f32>(offset);

    // Sample from 3D LUT
    let graded_color = textureSample(lut_texture, lut_sampler, lut_coord);

    // Blend between original and graded color based on intensity
    let final_color = mix(input_color.rgb, graded_color.rgb, uniforms.intensity);

    return vec4<f32>(final_color, input_color.a);
}

// Alternative fragment shader with better interpolation
@fragment
fn fs_main_trilinear(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Sample input color
    let input_color = textureSample(input_texture, input_sampler, uv);

    // Clamp to 0-1 range
    let rgb = clamp(input_color.rgb, vec3<f32>(0.0), vec3<f32>(1.0));

    // Manual trilinear interpolation for better quality
    let size = uniforms.lut_size;
    let scaled = rgb * (size - 1.0);
    let base = floor(scaled);
    let fract = scaled - base;

    // Calculate 8 corner coordinates for trilinear interpolation
    let c000 = (base + vec3<f32>(0.0, 0.0, 0.0) + vec3<f32>(0.5)) / size;
    let c100 = (base + vec3<f32>(1.0, 0.0, 0.0) + vec3<f32>(0.5)) / size;
    let c010 = (base + vec3<f32>(0.0, 1.0, 0.0) + vec3<f32>(0.5)) / size;
    let c110 = (base + vec3<f32>(1.0, 1.0, 0.0) + vec3<f32>(0.5)) / size;
    let c001 = (base + vec3<f32>(0.0, 0.0, 1.0) + vec3<f32>(0.5)) / size;
    let c101 = (base + vec3<f32>(1.0, 0.0, 1.0) + vec3<f32>(0.5)) / size;
    let c011 = (base + vec3<f32>(0.0, 1.0, 1.0) + vec3<f32>(0.5)) / size;
    let c111 = (base + vec3<f32>(1.0, 1.0, 1.0) + vec3<f32>(0.5)) / size;

    // Sample all 8 corners
    let v000 = textureSample(lut_texture, lut_sampler, c000).rgb;
    let v100 = textureSample(lut_texture, lut_sampler, c100).rgb;
    let v010 = textureSample(lut_texture, lut_sampler, c010).rgb;
    let v110 = textureSample(lut_texture, lut_sampler, c110).rgb;
    let v001 = textureSample(lut_texture, lut_sampler, c001).rgb;
    let v101 = textureSample(lut_texture, lut_sampler, c101).rgb;
    let v011 = textureSample(lut_texture, lut_sampler, c011).rgb;
    let v111 = textureSample(lut_texture, lut_sampler, c111).rgb;

    // Trilinear interpolation
    let v00 = mix(v000, v100, fract.x);
    let v01 = mix(v001, v101, fract.x);
    let v10 = mix(v010, v110, fract.x);
    let v11 = mix(v011, v111, fract.x);

    let v0 = mix(v00, v10, fract.y);
    let v1 = mix(v01, v11, fract.y);

    let graded_color = mix(v0, v1, fract.z);

    // Blend based on intensity
    let final_color = mix(input_color.rgb, graded_color, uniforms.intensity);

    return vec4<f32>(final_color, input_color.a);
}
