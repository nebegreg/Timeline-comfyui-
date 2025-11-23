use crate::{PluginContext, PluginError, PluginResult, ResourceLimits};
use anyhow::Result;
use wasmtime::*;

/// WASM runtime for executing WebAssembly plugins
pub struct WasmRuntime {
    engine: Engine,
    linker: Linker<WasmPluginState>,
    resource_limits: ResourceLimits,
}

/// State passed to WASM plugin instances
pub struct WasmPluginState {
    pub context: PluginContext,
    pub logs: Vec<String>,
    pub memory_limit: usize,
    pub memory_used: usize,
}

impl ResourceLimiter for WasmPluginState {
    fn memory_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool, anyhow::Error> {
        // Check if new size would exceed limit
        if desired > self.memory_limit {
            return Ok(false); // Deny the allocation
        }

        self.memory_used = desired;
        Ok(true) // Allow the allocation
    }

    fn table_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool, anyhow::Error> {
        // Allow reasonable table sizes (for function references, etc.)
        Ok(desired < 10_000)
    }

    fn instances(&self) -> usize {
        1 // Only one instance at a time
    }

    fn tables(&self) -> usize {
        1
    }

    fn memories(&self) -> usize {
        1
    }
}

impl WasmRuntime {
    pub fn new(resource_limits: ResourceLimits) -> Result<Self> {
        let mut config = Config::new();

        // Enable WASM features
        config.wasm_simd(true);
        config.wasm_bulk_memory(true);
        config.wasm_multi_value(true);
        config.wasm_reference_types(true);

        // Enable fuel metering for CPU limits
        config.consume_fuel(true);

        // Enable memory limits
        config.max_wasm_stack(256 * 1024); // 256KB stack limit

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        // Add custom host functions
        Self::define_host_functions(&mut linker)?;

        Ok(Self {
            engine,
            linker,
            resource_limits,
        })
    }

    pub fn with_default_limits() -> Result<Self> {
        Self::new(ResourceLimits::default())
    }

    pub fn execute_plugin(&self, module: &Module, context: PluginContext) -> Result<PluginResult> {
        let mut store = Store::new(
            &self.engine,
            WasmPluginState {
                context: context.clone(),
                logs: Vec::new(),
                memory_limit: self.resource_limits.max_memory_bytes as usize,
                memory_used: 0,
            },
        );

        // Set fuel for execution limits (fuel units roughly correlate to instructions)
        let fuel_amount = self.resource_limits.max_cpu_time_ms * 1_000_000; // ~1M fuel per ms
        store.set_fuel(fuel_amount)?;

        // Limit memory
        store.limiter(|state| state as &mut dyn ResourceLimiter);

        let instance = self.linker.instantiate(&mut store, module)?;

        // Call the plugin's main function
        let main_func = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_main")
            .map_err(|_| PluginError::WasmRuntime("plugin_main function not found".to_string()))?;

        let result_code = main_func.call(&mut store, ()).map_err(|e| {
            // Check if fuel exhausted (get_fuel returns remaining fuel)
            let remaining_fuel = store.get_fuel().unwrap_or(0);
            if remaining_fuel == 0 {
                PluginError::Timeout("WASM plugin exceeded CPU time limit".to_string())
            } else {
                PluginError::WasmRuntime(format!("Plugin execution failed: {}", e))
            }
        })?;

        let state = store.data();
        let success = result_code == 0;

        Ok(PluginResult {
            success,
            output_items: vec![], // TODO: Extract from WASM memory if needed
            modified_sequence: None,
            artifacts: vec![],
            logs: state.logs.clone(),
            error_message: if success {
                None
            } else {
                Some(format!("Plugin returned error code: {}", result_code))
            },
        })
    }

    fn define_host_functions(linker: &mut Linker<WasmPluginState>) -> Result<()> {
        // Log function
        linker.func_wrap(
            "env",
            "log",
            |mut caller: Caller<'_, WasmPluginState>,
             ptr: i32,
             len: i32|
             -> Result<(), anyhow::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("Failed to get memory"))?;

                let log_str = {
                    let data = memory.data(&caller);
                    let log_bytes = &data[ptr as usize..(ptr + len) as usize];
                    String::from_utf8_lossy(log_bytes).to_string()
                };

                caller.data_mut().logs.push(log_str);
                Ok(())
            },
        )?;

        // Get current frame
        linker.func_wrap(
            "env",
            "get_current_frame",
            |caller: Caller<'_, WasmPluginState>| caller.data().context.current_frame,
        )?;

        // Get sequence width
        linker.func_wrap("env", "get_width", |caller: Caller<'_, WasmPluginState>| {
            caller.data().context.width as i32
        })?;

        // Get sequence height
        linker.func_wrap(
            "env",
            "get_height",
            |caller: Caller<'_, WasmPluginState>| caller.data().context.height as i32,
        )?;

        // Memory allocation (simple bump allocator)
        linker.func_wrap(
            "env",
            "alloc",
            |mut _caller: Caller<'_, WasmPluginState>, _size: i32| -> i32 {
                // Simple allocation - in a real implementation, you'd want a proper allocator
                // For now, just return a fixed offset (this is a stub)
                // In practice, you'd implement a proper WASM allocator
                1024 // Return offset in WASM memory
            },
        )?;

        Ok(())
    }
}

/// Helper functions for WASM plugin development
pub mod wasm_helpers {
    /// Generate a basic WASM plugin template in Rust
    pub fn generate_wasm_plugin_template(plugin_name: &str) -> String {
        format!(
            r#"//! {plugin_name} - A WASM plugin for Gausian Native Editor

use std::ffi::CStr;
use std::os::raw::{{c_char, c_int}};

// Host function bindings
extern "C" {{
    fn log(ptr: *const c_char, len: c_int);
    fn get_current_frame() -> i64;
    fn get_width() -> c_int;
    fn get_height() -> c_int;
    fn alloc(size: c_int) -> *mut u8;
}}

// Helper function to log messages
fn plugin_log(message: &str) {{
    unsafe {{
        log(message.as_ptr() as *const c_char, message.len() as c_int);
    }}
}}

// Main plugin entry point
#[no_mangle]
pub extern "C" fn plugin_main() -> c_int {{
    plugin_log("Plugin {plugin_name} starting");
    
    let current_frame = unsafe {{ get_current_frame() }};
    let width = unsafe {{ get_width() }};
    let height = unsafe {{ get_height() }};
    
    plugin_log(&format!("Processing frame {{}} ({{}}x{{}})", current_frame, width, height));
    
    // TODO: Implement your plugin logic here
    
    plugin_log("Plugin {plugin_name} completed successfully");
    
    0 // Return 0 for success
}}

// Plugin metadata
#[no_mangle]
pub extern "C" fn plugin_get_name() -> *const c_char {{
    "{plugin_name}\0".as_ptr() as *const c_char
}}

#[no_mangle]
pub extern "C" fn plugin_get_version() -> *const c_char {{
    "1.0.0\0".as_ptr() as *const c_char
}}
"#,
            plugin_name = plugin_name
        )
    }

    /// Generate Cargo.toml for a WASM plugin
    pub fn generate_wasm_cargo_toml(plugin_name: &str) -> String {
        format!(
            r#"[package]
name = "{plugin_name}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Add your dependencies here

[profile.release]
lto = true
opt-level = "s"  # Optimize for size
panic = "abort"
"#,
            plugin_name = plugin_name.to_lowercase().replace(' ', "-")
        )
    }
}
