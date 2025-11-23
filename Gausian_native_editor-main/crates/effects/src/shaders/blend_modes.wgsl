// Blend Modes shader
// Phase 2: Rich Effects & Transitions
// Implements 12+ industry-standard blend modes

@group(0) @binding(0) var base_texture: texture_2d<f32>;
@group(0) @binding(1) var blend_texture: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct Uniforms {
    blend_mode: u32,  // 0=Normal, 1=Multiply, 2=Screen, etc.
    opacity: f32,     // 0-1
    padding1: f32,
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

    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;

    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return output;
}

// Blend mode functions
fn blend_normal(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return blend;
}

fn blend_multiply(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return base * blend;
}

fn blend_screen(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(1.0) - (vec3<f32>(1.0) - base) * (vec3<f32>(1.0) - blend);
}

fn blend_overlay(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    var result: vec3<f32>;
    for (var i = 0; i < 3; i++) {
        if (base[i] < 0.5) {
            result[i] = 2.0 * base[i] * blend[i];
        } else {
            result[i] = 1.0 - 2.0 * (1.0 - base[i]) * (1.0 - blend[i]);
        }
    }
    return result;
}

fn blend_soft_light(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return (vec3<f32>(1.0) - 2.0 * blend) * base * base + 2.0 * blend * base;
}

fn blend_hard_light(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return blend_overlay(blend, base);  // Hard light is overlay with swapped arguments
}

fn blend_color_dodge(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    var result: vec3<f32>;
    for (var i = 0; i < 3; i++) {
        if (blend[i] >= 1.0) {
            result[i] = 1.0;
        } else {
            result[i] = min(1.0, base[i] / (1.0 - blend[i]));
        }
    }
    return result;
}

fn blend_color_burn(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    var result: vec3<f32>;
    for (var i = 0; i < 3; i++) {
        if (blend[i] <= 0.0) {
            result[i] = 0.0;
        } else {
            result[i] = max(0.0, 1.0 - (1.0 - base[i]) / blend[i]);
        }
    }
    return result;
}

fn blend_darken(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return min(base, blend);
}

fn blend_lighten(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return max(base, blend);
}

fn blend_difference(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return abs(base - blend);
}

fn blend_exclusion(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return base + blend - 2.0 * base * blend;
}

fn blend_add(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    return min(base + blend, vec3<f32>(1.0));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(base_texture, tex_sampler, input.uv);
    let blend = textureSample(blend_texture, tex_sampler, input.uv);

    var result: vec3<f32>;

    // Select blend mode
    switch uniforms.blend_mode {
        case 0u: { result = blend_normal(base.rgb, blend.rgb); }
        case 1u: { result = blend_multiply(base.rgb, blend.rgb); }
        case 2u: { result = blend_screen(base.rgb, blend.rgb); }
        case 3u: { result = blend_overlay(base.rgb, blend.rgb); }
        case 4u: { result = blend_soft_light(base.rgb, blend.rgb); }
        case 5u: { result = blend_hard_light(base.rgb, blend.rgb); }
        case 6u: { result = blend_color_dodge(base.rgb, blend.rgb); }
        case 7u: { result = blend_color_burn(base.rgb, blend.rgb); }
        case 8u: { result = blend_darken(base.rgb, blend.rgb); }
        case 9u: { result = blend_lighten(base.rgb, blend.rgb); }
        case 10u: { result = blend_difference(base.rgb, blend.rgb); }
        case 11u: { result = blend_exclusion(base.rgb, blend.rgb); }
        case 12u: { result = blend_add(base.rgb, blend.rgb); }
        default: { result = base.rgb; }
    }

    // Mix with opacity
    result = mix(base.rgb, result, uniforms.opacity);

    // Combine with alpha
    let alpha = mix(base.a, blend.a, uniforms.opacity);

    return vec4<f32>(result, alpha);
}
