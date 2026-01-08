# Lluvia-rs AI Development Instructions

## Project Overview
Lluvia-rs is a multi-target Rust workspace that provides GPU compute capabilities and media processing tools, compiled to both native and WebAssembly targets. The project bridges GPU computing (via wgpu) with media container parsing, specifically Transport Streams.

## Specifications

- `lluvia_media` uses the Transport Stream (TS) specification described in `local_references/ISO_IEC_13818-1_2023(en).pdf`.

## Architecture Components

### Core Crates Structure
- **`lluvia_gpu/`**: WebGPU compute pipeline abstraction with Session → ComputeNode → Buffer workflow
- **`lluvia_media/`**: Media container parsing (Transport Stream packets), focused on zero-copy byte manipulation
- **`pkg/`**: Generated WASM artifacts for both crates, targeting browser and Node.js environments

### Key Design Patterns

#### GPU Compute Flow (lluvia_gpu)
```rust
// Session-based resource management pattern used throughout
let session = Session::new().await?;
let shader_module = session.create_shader_module(&desc);
let compute_node = session.create_compute_node(&ComputeNodeDescriptor { ... });
let buffer = session.create_buffer(size, usage, label);
session.run_command_buffer(&command_buffer);
```

#### Media Processing Pattern (lluvia_media)
```rust
// Zero-copy slice-based parsing with bytes::Bytes
let packet_slice = data.slice(i * PACKET_SIZE..(i + 1) * PACKET_SIZE);
let packet_view = PacketView::new(packet_slice);
```

#### Error Handling Convention
- Use `thiserror::Error` for domain-specific errors (`SessionError`, `TsError`)
- Async functions return `Result<T, SpecificError>`, not generic anyhow
- WASM-compatible error handling throughout

## Build & Test Workflows

### WASM Development
```bash
# Build both crates to pkg/ directory
wasm-pack build --out-dir '../../pkg' crates/lluvia_gpu
wasm-pack build --out-dir '../../pkg' crates/lluvia_media

# Test with WebGPU features enabled
wasm-pack test --headless --firefox crates/lluvia_gpu
cd crates/lluvia_media && wasm-pack test --node
```

### Browser Testing Requirements
- Chrome/Chromium with `--enable-unsafe-webgpu --enable-webgpu-developer-features`
- Firefox with fake media stream prefs (see `webdriver.json`)
- Tests use `wasm_bindgen_test` with `#[wasm_bindgen_test(unsupported = test)]` pattern

### Python Integration
- Uses maturin build system (`pyproject.toml` at root)
- Python environment setup requires pyenv 3.12.9 + specific apt dependencies
- Build targets both Python extensions and WASM modules

## Code Conventions

### Module Organization
- Flat re-exports in `lib.rs`: `pub use module_name::*;`
- Test files parallel source structure: `tests/test_[module].rs`
- Builder pattern via `bon::Builder` for complex descriptors

### WebGPU Integration Specifics
- All GPU resources created through `Session` (device/queue wrapper)
- Port-based binding system for compute shader parameters (`PortDescriptor`)
- Command encoder → command buffer → queue submission pattern
- Error scoping with `device.push_error_scope()` for validation

### Transport Stream Processing
- Fixed 188-byte packet size (`PACKET_SIZE` constant)
- Byte-level packet validation and parsing
- Memory-mapped file reading via `TsMemoryReader`

## Dependencies & Environment
- Core: wgpu 24.x, tokio for async, bytes for zero-copy operations
- WASM: wasm-bindgen ecosystem, supports both browser and Node.js
- Python: maturin + PyO3 (configured but implementation TBD)
- Development requires GStreamer tools for sample data generation

## Testing Approach
- GPU tests require actual hardware/browser WebGPU support
- Media parsing tests use embedded sample data for deterministic results
- WASM tests run in browser via wasm-pack test infrastructure
- Use `rstest` for parameterized testing where applicable