use glam::Vec3;
use wgpu::util::DeviceExt;

#[repr(C)]
pub struct Vertex {
    position: Vec3,
}
impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}
pub fn make_mesh(device: &wgpu::Device) -> Mesh {
    // Use mesh to avoid Vertexes duplicates (quad has one adjacent side- 2 vertices) with the 2 triangles composing it
    let vertices: [Vertex; 4] = [
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
        },
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.0),
        },
        Vertex {
            position: Vec3::new(-0.5, 0.5, 0.0),
        },
        Vertex {
            position: Vec3::new(0.5, 0.5, 0.0),
        },
    ];
    let indices = [0, 1, 2, 2, 1, 3]; // drawing order of each index (counter-clockwise)

    let mut content_bytes = unsafe { any_as_u8_slice(&vertices) };
    let mut buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("mesh vertex buffer descriptor"),
        contents: content_bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };
    let vertex_buffer = device.create_buffer_init(&buffer_descriptor);

    content_bytes = unsafe { any_as_u8_slice(&indices) };
    buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("mesh index buffer descriptor"),
        contents: content_bytes,
        usage: wgpu::BufferUsages::INDEX,
    };
    let index_buffer = device.create_buffer_init(&buffer_descriptor);

    return Mesh {
        vertex_buffer,
        index_buffer,
    };
}
