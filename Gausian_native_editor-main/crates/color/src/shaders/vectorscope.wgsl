// Vectorscope Compute Shader
// Phase 3: Advanced Color Management & LUTs
//
// Generates a vectorscope display showing chrominance (U/V) distribution
// in a circular plot centered at (0.5, 0.5)

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<u32>;

// Convert RGB to YUV (Rec. 709)
fn rgb_to_yuv(rgb: vec3<f32>) -> vec3<f32> {
    let y = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let u = (rgb.b - y) / 1.8556;
    let v = (rgb.r - y) / 1.5748;
    return vec3<f32>(y, u, v);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_dims = textureDimensions(input_texture);
    let output_width = 512u;
    let output_height = 512u;

    // Clear output buffer first
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
    let yuv = rgb_to_yuv(pixel.rgb);

    // Map U/V to output coordinates
    // U and V range from approximately -0.5 to 0.5
    // Map to 0-1 range for output
    let u_normalized = clamp(yuv.y + 0.5, 0.0, 1.0);
    let v_normalized = clamp(yuv.z + 0.5, 0.0, 1.0);

    let output_x = u32(u_normalized * f32(output_width - 1u));
    let output_y = u32((1.0 - v_normalized) * f32(output_height - 1u)); // Invert Y

    // Accumulate in output buffer
    let output_idx = output_y * output_width + output_x;
    if output_idx < output_width * output_height {
        atomicAdd(&output_buffer[output_idx], 1u);
    }
}
