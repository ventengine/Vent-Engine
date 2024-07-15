#version 450 core
layout(location = 0) out vec4 color;
layout(set = 0, binding = 0) uniform sampler2D sTexture;
layout(location = 0) in struct {
    vec4 Color;
    vec2 UV;
} In;
void main() {
    color = vec4(255); //In.Color * texture(sTexture, In.UV.st)
}