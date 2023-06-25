struct InstanceInput {
    @location(5) model_matrix0: vec4<f32>,
    @location(6) model_matrix1: vec4<f32>,
    @location(7) model_matrix2: vec4<f32>,
    @location(8) model_matrix3: vec4<f32>
}

struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Pose {
    data: array<mat4x4<f32>, 120>
}

@group(2) @binding(0)
var<uniform> animated_pose: Pose;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) weights: vec4<f32>,
    @location(4) joints: vec4<u32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var skin: mat4x4<f32> = animated_pose.data[model.joints.x] * model.weights.x
                            + animated_pose.data[model.joints.y] * model.weights.y
                            + animated_pose.data[model.joints.z] * model.weights.z
                            + animated_pose.data[model.joints.w] * model.weights.w;
    let model_matrix = mat4x4<f32>(instance.model_matrix0,
                                   instance.model_matrix1,
                                   instance.model_matrix2,
                                   instance.model_matrix3);
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * skin * vec4<f32>(model.position, 1.0);
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse,  in.tex_coords);
}