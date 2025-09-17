use glm::*;
use wgpu::{naga::proc::index, util::DeviceExt};

#[repr(C)]
pub struct Vertex {
    position: Vec3,
    tex_coord: [f32; 2],
}

/*
For Index Buffer optimization
*/
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub fn make_quad(device: &wgpu::Device) -> Mesh {
    // Use mesh to avoid Vertexes duplicates (quad has one adjacent side- 2 vertices) with the 2 triangles composing it
    let vertices: [Vertex; 4] = [
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
            tex_coord: [0.0, 0.0], // bottom-left
        },
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.0),
            tex_coord: [1.0, 0.0], // bottom-right
        },
        Vertex {
            position: Vec3::new(-0.5, 0.5, 0.0),
            tex_coord: [0.0, 1.0], // top-left
        },
        Vertex {
            position: Vec3::new(0.5, 0.5, 0.0),
            tex_coord: [1.0, 1.0], // top-right
        },
    ];
    let indices = [0, 1, 2, 2, 1, 3]; // drawing order of each index (counter-clockwise)

    let mut contents_bytes = unsafe { any_as_u8_slice(&vertices) };
    let mut buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad vertex buffer"),
        contents: contents_bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };
    let vertex_buffer = device.create_buffer_init(&buffer_descriptor);

    contents_bytes = unsafe { any_as_u8_slice(&indices) };
    buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad index buffer"),
        contents: contents_bytes,
        usage: wgpu::BufferUsages::INDEX,
    };
    let index_buffer = device.create_buffer_init(&buffer_descriptor);
    return Mesh {
        vertex_buffer: vertex_buffer,
        index_buffer: index_buffer,
    };
}
