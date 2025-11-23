// Film Grain shader
// Phase 2: Rich Effects & Transitions

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct Uniforms {
    intensity: f32,
    size: f32,
    seed: f32,
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

    // Full-screen quad
    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;

    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return output;
}

// Procedural noise function (hash-based)
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Smoothstep for smooth interpolation
    let u = f * f * (3.0 - 2.0 * f);

    // Sample 4 corners
    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    // Bilinear interpolation
    return mix(
        mix(a, b, u.x),
        mix(c, d, u.x),
        u.y
    );
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(input_texture, tex_sampler, input.uv);

    // Generate grain noise
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let scaled_uv = input.uv * texture_size / uniforms.size;

    // Add seed for variation
    let seeded_uv = scaled_uv + vec2<f32>(uniforms.seed);

    // Generate noise value (-1 to 1 range)
    let grain = (noise(seeded_uv) - 0.5) * 2.0;

    // Apply grain with intensity
    let grain_strength = grain * uniforms.intensity;

    // Add grain to all color channels
    var grainy = color.rgb + vec3<f32>(grain_strength);

    // Clamp to valid range
    grainy = clamp(grainy, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(grainy, color.a);
}
