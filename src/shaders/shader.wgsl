@group(0) @binding(0) var material_texture: texture_2d<f32>;
@group(0) @binding(1) var material_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) texture_coords: vec2<f32>
}

struct InstanceInput {
    @location(5) vec_1 : vec4<f32>,
    @location(6) vec_2 : vec4<f32>,
    @location(7) vec_3 : vec4<f32>,
    @location(8) vec_4 : vec4<f32>,
}

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>
}


@vertex
fn vs_main(vertex: Vertex, instance_input: InstanceInput) -> VertexPayload {
    var out: VertexPayload;

    var instance_matrix = mat4x4<f32> (
        instance_input.vec_1,
        instance_input.vec_2,
        instance_input.vec_3,
        instance_input.vec_4,
    );
    out.position = instance_matrix * vec4<f32>(vertex.position, 1.0);
    out.texture_coords = vertex.texture_coords;
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    return textureSample(material_texture, material_sampler, in.texture_coords);
}