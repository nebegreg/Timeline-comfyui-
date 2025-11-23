//! Simple WASM Effect Template
//!
//! This is a template for creating WASM-based video effect plugins
//! for Gausian Native Editor.
//!
//! To build this plugin:
//! ```bash
//! cargo build --target wasm32-wasi --release
//! cp target/wasm32-wasi/release/simple_effect.wasm ../simple_effect.wasm
//! ```

use std::os::raw::{c_char, c_int};

// Host function bindings (provided by Gausian Native Editor)
extern "C" {
    fn log(ptr: *const c_char, len: c_int);
    fn get_current_frame() -> i64;
    fn get_width() -> c_int;
    fn get_height() -> c_int;
}

/// Helper function to log messages from plugin
fn plugin_log(message: &str) {
    unsafe {
        log(message.as_ptr() as *const c_char, message.len() as c_int);
    }
}

/// Main plugin entry point
/// Returns 0 for success, non-zero for error
#[no_mangle]
pub extern "C" fn plugin_main() -> c_int {
    plugin_log("Simple WASM Effect Plugin starting...");

    // Get context from host
    let current_frame = unsafe { get_current_frame() };
    let width = unsafe { get_width() };
    let height = unsafe { get_height() };

    plugin_log(&format!(
        "Processing frame {} ({}x{})",
        current_frame, width, height
    ));

    // In a real plugin, you would:
    // 1. Read frame data from shared memory or WASI file
    // 2. Apply your effect (e.g., color transformation, blur, etc.)
    // 3. Write processed frame data back
    // 4. Return success/error code

    // Example effect: Log some info
    plugin_log("Applying simple effect transformation...");

    // Simulate some processing
    let pixel_count = width * height;
    plugin_log(&format!("Processing {} pixels", pixel_count));

    plugin_log("Simple WASM Effect completed successfully");

    0 // Return 0 for success
}

/// Get plugin name
#[no_mangle]
pub extern "C" fn plugin_get_name() -> *const c_char {
    "Simple WASM Effect\0".as_ptr() as *const c_char
}

/// Get plugin version
#[no_mangle]
pub extern "C" fn plugin_get_version() -> *const c_char {
    "1.0.0\0".as_ptr() as *const c_char
}
