#version 450 core
layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;
layout(push_constant) uniform uPushConstant {
    vec2 uScale;
    vec2 uTranslate;
} pc;

out gl_PerVertex {
    vec4 gl_Position;
};
layout(location = 0) out struct {
    vec4 Color;
    vec2 UV;
} Out;

void main() {
    Out.Color = color;
    Out.UV = uv;
    gl_Position = vec4(pos * pc.uScale + pc.uTranslate , 0, 1);
}