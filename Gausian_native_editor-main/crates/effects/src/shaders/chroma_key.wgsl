// Chroma Key shader (green screen keying)
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    key_color: vec3<f32>,     // RGB color to key out
    padding1: f32,
    tolerance: f32,           // Color distance threshold
    edge_feather: f32,        // Edge softness
    spill_suppression: f32,   // Spill reduction strength
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

// RGB to YUV conversion (Rec.709)
fn rgb_to_yuv(rgb: vec3<f32>) -> vec3<f32> {
    let y = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
    let u = (rgb.b - y) / 1.8556;
    let v = (rgb.r - y) / 1.5748;
    return vec3<f32>(y, u, v);
}

// YUV to RGB conversion (Rec.709)
fn yuv_to_rgb(yuv: vec3<f32>) -> vec3<f32> {
    let r = yuv.x + 1.5748 * yuv.z;
    let g = yuv.x - 0.1873 * yuv.y - 0.4681 * yuv.z;
    let b = yuv.x + 1.8556 * yuv.y;
    return vec3<f32>(r, g, b);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(input_texture, tex_sampler, input.uv);

    // Convert to YUV for better keying
    let pixel_yuv = rgb_to_yuv(color.rgb);
    let key_yuv = rgb_to_yuv(uniforms.key_color);

    // Calculate color distance in YUV space (emphasize chroma)
    let y_diff = abs(pixel_yuv.x - key_yuv.x);
    let u_diff = abs(pixel_yuv.y - key_yuv.y);
    let v_diff = abs(pixel_yuv.z - key_yuv.z);

    // Weighted distance (chroma is more important)
    let distance = sqrt(
        y_diff * y_diff * 0.5 +
        u_diff * u_diff * 1.5 +
        v_diff * v_diff * 1.5
    );

    // Calculate alpha with edge feathering
    var alpha = smoothstep(
        uniforms.tolerance - uniforms.edge_feather,
        uniforms.tolerance + uniforms.edge_feather,
        distance
    );

    // Spill suppression (reduce key color in non-keyed areas)
    if (alpha > 0.1 && uniforms.spill_suppression > 0.0) {
        // Neutralize the key color component
        var yuv_corrected = pixel_yuv;

        // Reduce the chroma components toward neutral
        let spill_factor = (1.0 - alpha) * uniforms.spill_suppression;
        yuv_corrected.y = mix(yuv_corrected.y, 0.0, spill_factor);
        yuv_corrected.z = mix(yuv_corrected.z, 0.0, spill_factor);

        let rgb_corrected = yuv_to_rgb(yuv_corrected);
        color = vec4<f32>(clamp(rgb_corrected, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
    }

    // Apply keyed alpha
    color.a *= alpha;

    return color;
}
