// Waveform Scope Compute Shader
// Phase 3: Advanced Color Management & LUTs
//
// Generates a waveform display showing luminance distribution
// across the horizontal position of the image

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<u32>;

// Convert RGB to luminance (Rec. 709)
fn rgb_to_luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_dims = textureDimensions(input_texture);
    let output_width = 512u;
    let output_height = 512u;

    // Clear output buffer first (only in first invocation)
    if global_id.x == 0u && global_id.y == 0u {
        for (var i = 0u; i < output_width * output_height; i++) {
            output_buffer[i] = 0u;
        }
    }

    workgroupBarrier();

    // Sample input texture
    if global_id.x >= input_dims.x || global_id.y >= input_dims.y {
        return;
    }

    let pixel = textureLoad(input_texture, vec2<i32>(global_id.xy), 0);
    let luma = rgb_to_luma(pixel.rgb);

    // Map horizontal position to output X
    let output_x = (global_id.x * output_width) / input_dims.x;

    // Map luminance (0-1) to output Y (inverted, 1.0 = top)
    let output_y = u32((1.0 - luma) * f32(output_height - 1u));

    // Accumulate in output buffer (RGBA format, using R channel for now)
    let output_idx = output_y * output_width + output_x;
    if output_idx < output_width * output_height {
        // Atomic add to handle concurrent writes
        atomicAdd(&output_buffer[output_idx], 1u);
    }
}
