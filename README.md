# videomti-render

**videomti-render** is a high-performance, modular video rendering engine built in Rust using `wgpu` (WebGPU). It is designed to be the "agnostic core" of a video editor, capable of compositing video, images, and colors into frames with high precision and low latency.

## 🚀 Features

*   **Pure Pipeline Architecture**: Decoupled frame description (`FrameDescription`) from the rendering execution.
*   **WGPU 0.28 Native**: Built for the latest WebGPU API, supporting modern GPU features and portability (Vulkan, Metal, DX12, OpenGL).
*   **Headless & Windowed**: First-class support for both invisible frame export (BufferSink) and real-time preview (SurfaceSink).
*   **Zero-Copy Texture Management**: Efficient handling of video frame uploads using `TextureManager` and `wgpu::Queue::write_texture`.
*   **Declarative Data Model**: simple, serializable structs to define compositions (`Layer`, `Transform`, `Opacity`).

## 🛠️ Architecture

The project is divided into distinct modules:

*   **`core`**: Context creation (`Instance`, `Device`, `Queue`) and error handling.
*   **`model`**: Data structures defining *what* to render (`Composition`, `Layer`).
*   **`resources`**: GPU resource management (`TextureManager`).
*   **`pipeline`**: The rendering logic (Shaders, BindGroups, `wgpu::RenderPipeline`).
*   **`renderer`**: The high-level orchestrator that executes the render pass.
*   **`outputs`**: Destinations for rendered pixels (`SurfaceSink`, `BufferSink`).

## 📦 Usage

### Prerequisites
*   Rust (latest stable)
*   A GPU (or software rasterizer via LLVMpipe/WARP if testing headless)

### Running Tests
The project includes an End-to-End integration test that renders a composition to a PNG file.

```bash
cargo test --test e2e_render
```
Check `output_audit.png` after running the test to verify the output.

## 📝 Example

```rust
use videomti_render::renderer::Renderer;

// 1. Initialize Context
let ctx = RenderContext::new(None).await?;

// 2. Load Resources
let mut texture_mgr = TextureManager::new();
texture_mgr.update_texture(..., &video_frame_data);

// 3. Define the Frame
let frame = FrameDescription {
    layers: vec![Layer { ... }],
    ...
};

// 4. Render!
renderer.render(&ctx, &texture_mgr, &frame, &mut sink)?;
```

## 📄 License
MIT License. See [LICENSE](LICENSE) for details.
