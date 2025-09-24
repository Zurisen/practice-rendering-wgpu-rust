# WGPU Render Practice

A Rust project for learning WGPU rendering with instanced geometry.

## WGPU Buffer Slots and Shader Locations Guide

This guide explains the relationship between physical GPU buffers, buffer slots, and shader locations in WGPU.

### Physical GPU Memory Layout

WGPU uses separate physical GPU buffers to store different types of vertex data:

#### GPU Buffer 1: Vertex Data

```
┌─────────────────────────────┐
│ GPU Buffer 1 (Vertex Data)  │ ← Created with device.create_buffer()
│ [pos_x, pos_y, pos_z,       │
│  tex_u, tex_v,              │
│  pos_x, pos_y, pos_z,       │
│  tex_u, tex_v, ...]         │
└─────────────────────────────┘
```

#### GPU Buffer 2: Instance Data

```
┌─────────────────────────────┐
│ GPU Buffer 2 (Instance Data)│ ← Created with device.create_buffer()
│ [m00, m01, m02, m03,        │
│  m10, m11, m12, m13,        │
│  m20, m21, m22, m23,        │
│  m30, m31, m32, m33, ...]   │
└─────────────────────────────┘
```

### Pipeline Configuration

Buffer slots act as logical binding points that connect physical GPU buffers to the render pipeline:

- **Buffer Slot 0** → Points to GPU Buffer 1 (Vertex Data)
- **Buffer Slot 1** → Points to GPU Buffer 2 (Instance Data)

```rust
// Binding buffers to slots
render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));   // Slot 0
render_pass.set_vertex_buffer(1, instance_buffer.slice(.)); // Slot 1
```

### Vertex Attribute Layout

The shader sees all vertex data through a unified location-based addressing system:

| Location       | Buffer Slot | Offset   | Data Description                       |
| -------------- | ----------- | -------- | -------------------------------------- |
| `@location(0)` | Slot 0      | 0 bytes  | Position from vertex buffer            |
| `@location(1)` | Slot 0      | 12 bytes | Texture coordinates from vertex buffer |
| `@location(5)` | Slot 1      | 0 bytes  | Matrix row 1 from instance buffer      |
| `@location(6)` | Slot 1      | 16 bytes | Matrix row 2 from instance buffer      |
| `@location(7)` | Slot 1      | 32 bytes | Matrix row 3 from instance buffer      |
| `@location(8)` | Slot 1      | 48 bytes | Matrix row 4 from instance buffer      |

### Shader Declaration

```wgsl
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,      // From vertex buffer
    @location(1) tex_coords: vec2<f32>,    // From vertex buffer
    @location(5) matrix_row_1: vec4<f32>,  // From instance buffer
    @location(6) matrix_row_2: vec4<f32>,  // From instance buffer
    @location(7) matrix_row_3: vec4<f32>,  // From instance buffer
    @location(8) matrix_row_4: vec4<f32>,  // From instance buffer
) -> VertexPayload {
    // Shader logic here
}
```

### Key Concepts

- **Physical Buffers**: Separate GPU memory allocations containing raw vertex data
- **Buffer Slots**: Logical attachment points (0, 1, 2...) where buffers are bound to the pipeline
- **Locations**: Unified addressing scheme (@location(0), @location(1)...) for vertex attributes in shaders
- **No Overlap**: Each location index must be unique across all buffer slots to avoid conflicts

The GPU's vertex fetcher reads from multiple physical buffers simultaneously and presents the data as a unified set of vertex attributes to your shader through the location system.

## Project Structure

```
src/
├── main.rs                     # Application entry point
├── renderer_backend/
│   ├── mod.rs                  # Module declarations
│   ├── state.rs               # Main render state
│   ├── pipeline_builder.rs    # Render pipeline construction
│   ├── mesh_builder.rs        # Vertex data generation
│   ├── instance.rs            # Instance data structures
│   └── material.rs            # Texture and material handling
├── shaders/
│   └── shader.wgsl            # WGSL vertex and fragment shaders
└── textures/
    └── some_texture.jpg    # Sample texture asset
```

## Running the Project

```bash
cargo build
cargo run
```

## Dependencies

- `wgpu` - Graphics API abstraction
- `glfw` - Window management
- `cgmath` - 3D math library
- `glam` - Linear algebra for graphics
- `bytemuck` - Safe transmutation between types
- `image` - Image loading and processing
