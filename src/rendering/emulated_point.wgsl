
struct CameraUniform{
    view_proj: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

//@group(0) @binding(1)
//var<uniform> point_size: f32 = 5.0;

//var<in> @builtin(instance_index) instance_idx: u32;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(instance_index) instance_idx : u32,
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    // Upper left pos
    var pos_x = model.position.x - 1.0;
    var pos_y = model.position.y - 1.0;
    if (instance_idx == 1u || instance_idx == 4u) {
        pos_x += 2.0; 
    }
    if (instance_idx == 2u || instance_idx == 3u) {
        pos_y += 2.0;
    }
    if (instance_idx == 5u) {
        pos_x += 2.0;
        pos_y += 2.0;
    }
    out.clip_position = camera.view_proj * vec4<f32>(pos_x, pos_y, model.position.z, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}