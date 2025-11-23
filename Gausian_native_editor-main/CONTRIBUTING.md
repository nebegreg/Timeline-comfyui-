# Contributing to Gausian Native Editor

Thank you for your interest in contributing! This document provides guidelines and architecture overview for contributors.

## 🏗️ Project Architecture

### Workspace Structure

```
Gausian_native_editor/
├── apps/
│   ├── desktop/          # Main egui application
│   └── comfywebview/     # ComfyUI WebView integration
│
├── crates/
│   ├── timeline/         # Timeline data structures & commands
│   ├── project/          # SQLite database & persistence
│   ├── media-io/         # FFmpeg/ffprobe integration
│   ├── renderer/         # WGPU GPU renderer
│   ├── exporters/        # FCPXML/EDL/FCP7 export
│   ├── jobs/             # Job queue system
│   ├── plugin-host/      # WASM/Python plugin runtime
│   ├── native-decoder/   # VideoToolbox/GStreamer decoding
│   ├── cli/              # CLI tool
│   ├── effects/          # GPU effects system (NEW - Phase 2)
│   └── color/            # Color management & LUTs (NEW - Phase 3)
│
├── ROADMAP_DETAILED.md   # Full roadmap with 8 phases
└── IMPLEMENTATION_PLAN.md # Implementation details
```

## 🚀 Current Development Status

### ✅ Phase 1: Timeline Polish & UX (IN PROGRESS)

New modules created:
- `apps/desktop/src/selection.rs` - Multi-clip selection
- `apps/desktop/src/edit_modes.rs` - Ripple/Roll/Slide/Slip modes
- `apps/desktop/src/keyboard.rs` - Professional shortcuts (J/K/L, I/O, etc.)
- `crates/timeline/src/markers.rs` - Markers and regions

**Next steps:**
- Integrate selection system into app.rs
- Wire up keyboard shortcuts
- Implement ripple edit logic
- UI for markers visualization

### 🔜 Phase 2: Rich Effects & Transitions (PREPARED)

Crate structure created:
- `crates/effects/` - Effect trait and manager
- `brightness_contrast.rs`, `blur.rs`, `transform.rs` - First 3 effects
- WGSL shader: `shaders/brightness_contrast.wgsl`

**To implement:**
- Complete GPU pipelines
- Add 12+ more effects
- Effect stack UI in inspector
- Transitions system

### 🔜 Phase 3: Color Management & LUTs (PREPARED)

Crate structure created:
- `crates/color/` - Color spaces and LUT system
- `lut3d.rs` - 3D LUT data structure
- `parsers/cube.rs` - Adobe .cube parser (functional)
- `color_spaces.rs` - sRGB, Rec709, Rec2020, ACES

**To implement:**
- Complete LUT GPU application shader
- Video scopes (waveform, vectorscope)
- ACES workflow
- UI for LUT management

## 🛠️ Development Setup

### Prerequisites

```bash
# Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# FFmpeg & GStreamer (macOS)
brew install ffmpeg gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-libav

# FFmpeg & GStreamer (Ubuntu/Debian)
sudo apt-get install -y ffmpeg gstreamer1.0-libav gstreamer1.0-plugins-{base,good,bad} gstreamer1.0-tools
```

### Building

```bash
# Build entire workspace
cargo build --release

# Run desktop app
cargo run --bin desktop

# Run CLI
cargo run -p cli -- --help

# Run tests
cargo test --workspace

# Linting
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

## 📝 Coding Guidelines

### Rust Style

- Follow official Rust style guide
- Use `cargo fmt` before committing
- Pass `cargo clippy` with zero warnings
- Write doc comments for public APIs

### Commit Messages

Follow conventional commits:

```
feat: Add multi-clip selection system
fix: Correct ripple edit offset calculation
docs: Update CONTRIBUTING.md with Phase 2 info
refactor: Extract marker logic to separate module
perf: Optimize timeline rendering with culling
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes with clear commits
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test --workspace`
6. Run clippy: `cargo clippy --workspace`
7. Submit PR with description of changes

## 🎯 How to Contribute

### Good First Issues

- **Phase 1**: Implement zoom-to-selection
- **Phase 1**: Add visual snap indicators
- **Phase 2**: Implement saturation/hue effect
- **Phase 2**: Create film grain shader
- **Phase 3**: Add .3dl LUT parser
- **Documentation**: Add examples to plugin SDK

### High-Impact Contributions

- **Phase 2**: Complete effect stack UI
- **Phase 3**: Video scopes implementation
- **Phase 4**: LORA training integration
- **Phase 7**: Collaborative editing backend

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_creation() {
        let marker = Marker::new(100, "Test".to_string());
        assert_eq!(marker.frame, 100);
        assert_eq!(marker.label, "Test");
    }
}
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run specific test
cargo test test_timeline_ripple_edit
```

## 📚 Documentation

- **API Docs**: `cargo doc --open --workspace`
- **User Manual**: See `docs/` (to be created)
- **Plugin SDK**: See `crates/plugin-host/README.md` (to be created)

## 🐛 Reporting Bugs

When filing an issue, include:

1. **Environment**: OS, Rust version, GPU info
2. **Steps to reproduce**
3. **Expected vs actual behavior**
4. **Logs**: Enable with `RUST_LOG=debug cargo run`
5. **Screenshots/videos** if UI-related

## 💡 Feature Requests

Before proposing features:

1. Check existing roadmap in `ROADMAP_DETAILED.md`
2. Search existing issues
3. Describe use case and benefits
4. Consider implementation complexity

## 🔐 Security

Report security issues privately to: security@gausian.xyz (if applicable)

## 📄 License

By contributing, you agree your contributions will be licensed under MPL-2.0.

---

## 🎓 Learning Resources

### Rust Video Editing

- **WGPU**: https://wgpu.rs/
- **egui**: https://www.egui.rs/
- **Video Processing**: FFmpeg documentation

### Color Science

- **ACES**: https://www.acescentral.com/
- **LUT Formats**: https://docs.acescentral.com/specifications/

### GPU Shaders

- **WGSL**: https://www.w3.org/TR/WGSL/
- **Learn WGPU**: https://sotrh.github.io/learn-wgpu/

---

**Questions?** Join our Discord: https://discord.gg/JfsKWDBXHT

**Happy coding! 🦀**
