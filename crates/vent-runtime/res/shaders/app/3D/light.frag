#version 460 core

layout(location = 1) in vec3 color;

layout(location = 0) out vec4 clip_position;

void main() {
    clip_position = vec4(color, 1.0);
}