# Simple WASM Effect Template

This is a template plugin for creating WASM-based video effects for Gausian Native Editor.

## Building

```bash
# Install WASM target if not already installed
rustup target add wasm32-wasi

# Build the plugin
cargo build --target wasm32-wasi --release

# Copy the built WASM module to the plugin directory
cp target/wasm32-wasi/release/simple_effect_wasm.wasm simple_effect.wasm
```

## Development

1. Modify `src/lib.rs` to implement your effect
2. Use the host functions to interact with the editor:
   - `log()` - Send log messages
   - `get_current_frame()` - Get current frame number
   - `get_width()` / `get_height()` - Get frame dimensions
3. Read/write frame data through WASI filesystem or shared memory
4. Return 0 for success, non-zero for errors

## Example Effects

You can implement various effects:
- Color corrections (brightness, contrast, saturation)
- Filters (blur, sharpen, edge detection)
- Transformations (rotate, scale, distort)
- Custom image processing algorithms

## Performance Tips

- Use `opt-level = "s"` for smaller WASM files
- Avoid allocations in hot loops
- Use SIMD operations when possible
- Profile with wasmtime's profiling tools
