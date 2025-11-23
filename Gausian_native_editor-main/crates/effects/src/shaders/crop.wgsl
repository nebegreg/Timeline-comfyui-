// Crop/Padding shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    feather: f32,
    padding1: f32,
    padding2: f32,
    padding3: f32,
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
    // Check if UV is within crop rectangle
    let in_x = input.uv.x >= uniforms.left && input.uv.x <= uniforms.right;
    let in_y = input.uv.y >= uniforms.top && input.uv.y <= uniforms.bottom;

    if (!in_x || !in_y) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);  // Transparent outside crop
    }

    // Calculate distance from crop edges for feathering
    var alpha = 1.0;

    if (uniforms.feather > 0.0) {
        let dist_left = input.uv.x - uniforms.left;
        let dist_right = uniforms.right - input.uv.x;
        let dist_top = input.uv.y - uniforms.top;
        let dist_bottom = uniforms.bottom - input.uv.y;

        let min_dist = min(min(dist_left, dist_right), min(dist_top, dist_bottom));

        // Smoothstep feathering
        alpha = smoothstep(0.0, uniforms.feather, min_dist);
    }

    var color = textureSample(input_texture, tex_sampler, input.uv);
    color.a *= alpha;

    return color;
}
