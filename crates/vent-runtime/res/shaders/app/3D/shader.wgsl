struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct UBO {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    transformation: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> ubo: UBO;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.position = ubo.projection * ubo.view * ubo.transformation * vec4(position, 1.0);
    return result;
}


@group(0)
@binding(0)
var texture_diffuse: texture_2d<f32>;
@group(0)
@binding(1)
var sampler_diffuse: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, sampler_diffuse, vertex.tex_coord);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}