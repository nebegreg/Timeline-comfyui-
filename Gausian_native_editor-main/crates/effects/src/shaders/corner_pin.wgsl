// Corner Pin Shader
// 4-point perspective transformation using bilinear interpolation
// Phase 2: Geometric Effects

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: CornerPinUniforms;

struct CornerPinUniforms {
    tl_x: f32,
    tl_y: f32,
    tr_x: f32,
    tr_y: f32,
    bl_x: f32,
    bl_y: f32,
    br_x: f32,
    br_y: f32,
    transform_matrix: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;

    // Full-screen triangle
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    output.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    output.tex_coords = vec2<f32>(x, y);

    return output;
}

// Bilinear interpolation of corner positions
// Given normalized screen coords (0-1), map to distorted quad
fn bilinear_transform(uv: vec2<f32>) -> vec2<f32> {
    // Get corner positions
    let tl = vec2<f32>(uniforms.tl_x, uniforms.tl_y);
    let tr = vec2<f32>(uniforms.tr_x, uniforms.tr_y);
    let bl = vec2<f32>(uniforms.bl_x, uniforms.bl_y);
    let br = vec2<f32>(uniforms.br_x, uniforms.br_y);

    // Bilinear interpolation
    // First interpolate top and bottom edges
    let top = mix(tl, tr, uv.x);
    let bottom = mix(bl, br, uv.x);

    // Then interpolate between top and bottom
    let result = mix(top, bottom, uv.y);

    return result;
}

// Inverse bilinear transformation (approximation)
// Maps distorted quad back to unit square for texture sampling
fn inverse_bilinear_transform(p: vec2<f32>) -> vec2<f32> {
    // This is a simplified inverse using Newton-Raphson iteration
    // For production, use proper homography inverse

    let tl = vec2<f32>(uniforms.tl_x, uniforms.tl_y);
    let tr = vec2<f32>(uniforms.tr_x, uniforms.tr_y);
    let bl = vec2<f32>(uniforms.bl_x, uniforms.bl_y);
    let br = vec2<f32>(uniforms.br_x, uniforms.br_y);

    // Newton-Raphson iteration to find (u,v) such that bilinear(u,v) = p
    var uv = vec2<f32>(0.5, 0.5);  // Initial guess

    for (var i = 0; i < 5; i = i + 1) {
        // Current estimate
        let top = mix(tl, tr, uv.x);
        let bottom = mix(bl, br, uv.x);
        let current = mix(top, bottom, uv.y);

        // Error
        let error = p - current;

        if (length(error) < 0.001) {
            break;
        }

        // Jacobian approximation (finite differences)
        let du = 0.01;
        let dv = 0.01;

        let top_u = mix(tl, tr, uv.x + du);
        let bottom_u = mix(bl, br, uv.x + du);
        let dx = mix(top_u, bottom_u, uv.y) - current;

        let top_v = mix(tl, tr, uv.x);
        let bottom_v = mix(bl, br, uv.x);
        let dy = mix(top_v, bottom_v, uv.y + dv) - current;

        // Update estimate
        uv = uv + vec2<f32>(
            dot(error, vec2<f32>(dx.x, dy.x)),
            dot(error, vec2<f32>(dx.y, dy.y))
        ) * 0.5;

        uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    }

    return uv;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Use inverse transform to find source texture coordinates
    let source_uv = inverse_bilinear_transform(input.tex_coords);

    // Check if UV is within valid range
    if (source_uv.x < 0.0 || source_uv.x > 1.0 ||
        source_uv.y < 0.0 || source_uv.y > 1.0) {
        // Outside bounds, return transparent
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Sample input texture
    let color = textureSample(input_tex, tex_sampler, source_uv);

    return color;
}
