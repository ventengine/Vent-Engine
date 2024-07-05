#version 450 core

layout(binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
} camera;

layout(binding = 1) uniform Light {
    vec3 position;
    vec3 color;
} light;

layout(location = 0) in vec3 in_position;

layout(location = 0) out vec4 clip_position;
layout(location = 1) out vec3 color;

const float scale = 1.0;

void main() {
    clip_position = camera.projection * camera.view * vec4(in_position * scale + light.position, 1.0);
    color = light.color;
}