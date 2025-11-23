// Separable Gaussian Blur shader
// Phase 2: Rich Effects & Transitions
//
// Two-pass blur: horizontal pass followed by vertical pass

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    radius: f32,
    direction: vec2<f32>,  // (1, 0) for horizontal, (0, 1) for vertical
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

// Gaussian kernel weights (9-tap)
// sigma = 2.0
const WEIGHTS = array<f32, 9>(
    0.0625,  // -4
    0.125,   // -3
    0.1875,  // -2
    0.25,    // -1
    0.3125,  //  0 (center)
    0.25,    //  1
    0.1875,  //  2
    0.125,   //  3
    0.0625   //  4
);

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let texel_size = 1.0 / texture_size;

    // Scale radius
    let pixel_radius = max(1.0, uniforms.radius);

    var color = vec4<f32>(0.0);
    var total_weight = 0.0;

    // Sample along direction with Gaussian weights
    let num_samples = 9;
    for (var i = 0; i < num_samples; i++) {
        let offset = f32(i - 4) * pixel_radius / 4.0;  // -4 to +4
        let sample_uv = input.uv + uniforms.direction * texel_size * offset;

        let sample_color = textureSample(input_texture, tex_sampler, sample_uv);
        let weight = WEIGHTS[i];

        color += sample_color * weight;
        total_weight += weight;
    }

    // Normalize
    if (total_weight > 0.0) {
        color /= total_weight;
    }

    return color;
}
