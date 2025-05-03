# Rust Multi-Window SDL2/WGPU/EGUI Template

<img width="918" alt="image" src="https://github.com/user-attachments/assets/e278e9bf-e7ab-4026-83cc-733060b7e876" />


A template project for building **multi-window** GUI applications in Rust using:

- [SDL2](https://github.com/Rust-SDL2/rust-sdl2) for windowing and input
- [WGPU](https://github.com/gfx-rs/wgpu) for GPU rendering
- [egui](https://github.com/emilk/egui) for the UI framework

## 🚀 Features

- Multi-window support via SDL2
- GPU-accelerated rendering using WGPU
- Immediate mode GUI with EGUI
- Event-driven architecture with window lifecycle management
- Easily extensible structure for prototyping or app development

## 🧱 Project Structure

```
src/
├── main.rs # Entry point and example application
├── gfx_util.rs # Framework for window/gfx context management, and a few helper functions
├── shader.wgsl # An example wgsl shader used by the example.
```

## 🧪 Build and Run

Clone and run:

```bash
git clone https://github.com/yourusername/rust-multiwindow-gfx-template.git
cd rust-multiwindow-template
cargo run
```
