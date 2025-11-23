// Spin Transition Shader
// 3D rotation transition with perspective
// Phase 2: Transitions System

@group(0) @binding(0) var from_frame: texture_2d<f32>;
@group(0) @binding(1) var to_frame: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> uniforms: SpinUniforms;

struct SpinUniforms {
    progress: f32,
    rotation_axis: f32,  // 0 = X, 1 = Y, 2 = Z
    direction: f32,
    perspective: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

const PI: f32 = 3.14159265359;

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

// 3D rotation matrices
fn rotate_y(angle: f32, p: vec3<f32>) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(
        c * p.x + s * p.z,
        p.y,
        -s * p.x + c * p.z
    );
}

fn rotate_x(angle: f32, p: vec3<f32>) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(
        p.x,
        c * p.y - s * p.z,
        s * p.y + c * p.z
    );
}

fn rotate_z(angle: f32, p: vec3<f32>) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(
        c * p.x - s * p.y,
        s * p.x + c * p.y,
        p.z
    );
}

// Apply perspective projection
fn apply_perspective(p: vec3<f32>, fov: f32) -> vec2<f32> {
    let depth = 1.0 + p.z * fov;
    return p.xy / depth;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.tex_coords;
    let center = vec2<f32>(0.5, 0.5);

    // Convert UV to centered 3D space
    var p = vec3<f32>(uv - center, 0.0);

    // Rotation angle based on progress
    // First half (0-0.5): from_frame rotates out
    // Second half (0.5-1.0): to_frame rotates in
    let half_progress = uniforms.progress * 2.0;

    var color: vec4<f32>;

    if (uniforms.progress < 0.5) {
        // First half: rotate from_frame out
        let angle = uniforms.progress * PI * uniforms.direction;

        var rotated_p: vec3<f32>;
        if (uniforms.rotation_axis < 0.5) {
            rotated_p = rotate_x(angle, p);
        } else if (uniforms.rotation_axis < 1.5) {
            rotated_p = rotate_y(angle, p);
        } else {
            rotated_p = rotate_z(angle, p);
        }

        // Apply perspective
        let projected = apply_perspective(rotated_p, uniforms.perspective);
        let sample_uv = projected + center;

        // Check bounds
        if (all(sample_uv >= vec2<f32>(0.0)) && all(sample_uv <= vec2<f32>(1.0))) {
            color = textureSample(from_frame, tex_sampler, sample_uv);
            // Fade out based on z-depth
            let fade = 1.0 - abs(rotated_p.z);
            color = color * fade;
        } else {
            // Out of bounds, show black (or to_frame in background)
            color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }

        // Blend with to_frame background
        let to_color = textureSample(to_frame, tex_sampler, uv);
        color = mix(to_color, color, color.a);

    } else {
        // Second half: rotate to_frame in
        let angle = (1.0 - uniforms.progress) * PI * uniforms.direction;

        var rotated_p: vec3<f32>;
        if (uniforms.rotation_axis < 0.5) {
            rotated_p = rotate_x(angle, p);
        } else if (uniforms.rotation_axis < 1.5) {
            rotated_p = rotate_y(angle, p);
        } else {
            rotated_p = rotate_z(angle, p);
        }

        // Apply perspective
        let projected = apply_perspective(rotated_p, uniforms.perspective);
        let sample_uv = projected + center;

        // Check bounds
        if (all(sample_uv >= vec2<f32>(0.0)) && all(sample_uv <= vec2<f32>(1.0))) {
            color = textureSample(to_frame, tex_sampler, sample_uv);
            // Fade in based on z-depth
            let fade = 1.0 - abs(rotated_p.z);
            color = color * fade;
        } else {
            color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
    }

    return color;
}
