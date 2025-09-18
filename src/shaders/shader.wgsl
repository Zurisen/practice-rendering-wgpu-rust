@group(0) @binding(0) var myTexture: texture_2d<f32>;
@group(0) @binding(1) var mySampler: sampler; 

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> camera: CameraUniform;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) texCoord: vec2<f32>
}

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec2<f32>
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};


@vertex
fn vs_main(vertex: Vertex, instance: InstanceInput) -> VertexPayload {

    var out: VertexPayload;
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    out.position = camera.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.texCoord = vertex.texCoord;
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    return textureSample(myTexture, mySampler, in.texCoord);
}