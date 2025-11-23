// Zoom Transition Shader
// Scale-based transition with smooth interpolation
// Phase 2: Transitions System

@group(0) @binding(0) var from_frame: texture_2d<f32>;
@group(0) @binding(1) var to_frame: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> uniforms: ZoomUniforms;

struct ZoomUniforms {
    progress: f32,
    zoom_direction: f32,  // 0 = zoom out, 1 = zoom in
    feather: f32,
    _padding: f32,
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

// Smooth step function
fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.tex_coords;
    let center = vec2<f32>(0.5, 0.5);

    // Calculate zoom scale based on progress
    // From frame zooms in (gets larger), to frame stays at original size
    let from_scale = 1.0 + uniforms.progress * 2.0;  // 1.0 to 3.0
    let to_scale = 1.0;

    // Transform UV coordinates for zoom
    let from_uv = (uv - center) / from_scale + center;
    let to_uv = uv;

    // Check if UV is in bounds for from_frame (after zoom, some areas go out of bounds)
    let from_in_bounds = all(from_uv >= vec2<f32>(0.0)) && all(from_uv <= vec2<f32>(1.0));

    // Sample textures
    var from_color = vec4<f32>(0.0);
    if (from_in_bounds) {
        from_color = textureSample(from_frame, tex_sampler, from_uv);
    }

    let to_color = textureSample(to_frame, tex_sampler, to_uv);

    // Mix based on progress with feathering
    let feather_start = 0.0;
    let feather_end = uniforms.feather;
    let alpha = smooth_step(feather_start, feather_end, uniforms.progress);

    // Additional fade out for from_frame when out of bounds
    var from_alpha = 1.0 - uniforms.progress;
    if (!from_in_bounds) {
        from_alpha = 0.0;
    }

    // Blend colors
    let result = mix(from_color.rgb, to_color.rgb, alpha);

    // Composite alpha
    let result_alpha = from_color.a * from_alpha + to_color.a * alpha;

    return vec4<f32>(result, max(result_alpha, to_color.a));
}
