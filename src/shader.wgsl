// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec4<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) light_dir: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_1: vec3<f32>,
    @location(10) normal_matrix_2: vec3<f32>,
    @location(11) normal_matrix_3: vec3<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> light: Light;

@vertex
fn vs_main(
    model: VertexInput, instance: InstanceInput
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_1,
        instance.normal_matrix_2,
        instance.normal_matrix_3,
    );

    let light_dir = vec3<f32>(1.0, 1.0, 1.0);

    var out: VertexOutput;

    out.tex_coords = model.tex_coords;
    out.world_pos = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * out.world_pos;
    // out.normal = (model_matrix * vec4<f32>(model.normal, 1.0)).xyz;
    // out.tangent = (model_matrix * vec4<f32>(model.tangent, 1.0)).xyz;
    // out.bitangent = (model_matrix * vec4<f32>(model.bitangent, 1.0)).xyz;
    out.world_normal = normal_matrix * model.normal;
    out.tangent = normal_matrix * model.tangent;
    out.bitangent = normal_matrix * model.bitangent;
    out.light_dir = (camera.view_proj * vec4<f32>(light_dir, 1.0)).xyz;

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@group(2) @binding(0)
var t_depth: texture_depth_2d;
@group(2) @binding(1)
var s_depth: sampler_comparison;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal_matrix = mat3x3<f32>(
        in.tangent,
        in.bitangent,
        in.world_normal,
    );
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    // let near = 0.1;
    // let far = 100.0;
    // return vec4<f32>(vec3<f32>(textureSampleCompare(t_depth, s_depth, in.tex_coords.xy, 0.0)), 1.0);

    let world_normal = normalize(normal_matrix * textureSample(t_normal, s_normal, in.tex_coords).xyz); // * dot(in.normal, light_dir);
    // let world_normal = in.world_normal;

    let light = light.position - in.world_pos.xyz;

    let half = normalize(light_dir - normalize(in.world_pos.xyz));

    // let diffuse_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let diffuse_color = vec4<f32>(0.5, 0.5, 0.5, 1.0);

    // let temp_normal = cross(in.tangent, in.bitangent);

    // let difference = normalize(temp_normal) - normalize(in.normal);
    // let diffuse_color = max(vec4<f32>(difference, 1.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));

    // let diffuse_color_lit = diffuse_color;

    let light_normal = normalize(light);

    // let diffuse_color_lit = diffuse_color * max(dot(world_normal, light_normal), 0.0); // * 10000. / (light.x * light.x + light.y * light.y + light.z * light.z), 0.0);

    // let c = camera.inv_view_proj * in.clip_position;

    let diffuse_strength = max(dot(world_normal, light_normal), 0.0);
    // let specular_strength = max(dot(half, world_normal), 0.0);
    let specular_strength = 0.0;
    let ambient_strength = 0.01;

    let color_lit = min(diffuse_strength + specular_strength + ambient_strength, 1.0) * diffuse_color;

    // let color = // return vec4<f32>(world_normal, 1.0);
    // return vec4<f32>(in.clip_position.xyw, 1.0);
    return color_lit;
    // return c;
}

