#version 450 core

layout (location = 0) in vec3 inPos;

layout(push_constant) uniform UBO 
{
	mat4 projection;
	mat4 model;
} ubo;

layout (location = 0) out vec3 outUVW;

void main() 
{
	outUVW = inPos;
	// Convert cubemap coordinates into Vulkan coordinate space
	outUVW.x = -outUVW.x;
	// Remove translation from view matrix
	mat4 viewMat = mat4(mat3(ubo.model));
	gl_Position = ubo.projection * viewMat * vec4(inPos.xyz, 1.0);
}