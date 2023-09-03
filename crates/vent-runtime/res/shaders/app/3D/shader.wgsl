struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

struct UBO {
    view_position: vec3<f32>,
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    transformation: mat4x4<f32>,
}

struct Material {
    base_color: vec4<f32>,
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> ubo: UBO;

@group(1) @binding(2) 
var<uniform> material : Material;

@group(2) @binding(0)
var<uniform> light: Light;


@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.normal = normal;
    result.world_position = position;
    result.position = ubo.projection * ubo.view * ubo.transformation * vec4(position, 1.0);
    return result;
}

@group(1)
@binding(0)
var texture_diffuse: texture_2d<f32>;
@group(1)
@binding(1)
var sampler_diffuse: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(texture_diffuse, sampler_diffuse, vertex.tex_coord) * material.base_color;

    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - vertex.world_position);
    let view_dir = normalize(ubo.view_position.xyz - vertex.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(vertex.world_position, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(vertex.world_position, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * sample.xyz;

    return vec4<f32>(result, sample.a);
}
