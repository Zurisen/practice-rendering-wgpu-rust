struct Vertex {
    @location(0) position: vec3<f32>
}

struct VertexPayload {
    @builtin(position) position: vec4<f32>
}

@vertex
fn vs_main(in: Vertex) -> VertexPayload {
    var out: VertexPayload;
    out.position = vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.5, 0.0, 1.0);
}