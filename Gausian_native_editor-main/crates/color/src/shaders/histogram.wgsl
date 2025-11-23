// Histogram Compute Shader
// Phase 3: Advanced Color Management & LUTs
//
// Generates RGB histogram showing distribution of color values
// Separate histograms for R, G, B channels

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<u32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_dims = textureDimensions(input_texture);
    let output_width = 512u;
    let output_height = 512u;
    let histogram_bins = 256u;

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

    // Convert RGB to 0-255 range
    let r_bin = u32(clamp(pixel.r, 0.0, 1.0) * f32(histogram_bins - 1u));
    let g_bin = u32(clamp(pixel.g, 0.0, 1.0) * f32(histogram_bins - 1u));
    let b_bin = u32(clamp(pixel.b, 0.0, 1.0) * f32(histogram_bins - 1u));

    // Map bins to output X coordinates
    // Use 3 rows: R (top), G (middle), B (bottom)
    let row_height = output_height / 3u;

    // Calculate output positions
    let r_x = (r_bin * output_width) / histogram_bins;
    let g_x = (g_bin * output_width) / histogram_bins;
    let b_x = (b_bin * output_width) / histogram_bins;

    // R histogram (top third)
    let r_idx = 0u * row_height * output_width + r_x;
    if r_idx < output_width * output_height {
        atomicAdd(&output_buffer[r_idx], 1u);
    }

    // G histogram (middle third)
    let g_idx = 1u * row_height * output_width + g_x;
    if g_idx < output_width * output_height {
        atomicAdd(&output_buffer[g_idx], 1u);
    }

    // B histogram (bottom third)
    let b_idx = 2u * row_height * output_width + b_x;
    if b_idx < output_width * output_height {
        atomicAdd(&output_buffer[b_idx], 1u);
    }
}
