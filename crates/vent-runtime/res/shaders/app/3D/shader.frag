#version 460 core

layout (binding = 0) uniform sampler2D texture_diffuse;

layout (binding = 1) uniform Material {
    vec4 base_color;
    int alpha_mode;
    float alpha_cutoff;
} material;


layout (binding = 2) uniform Light {
    vec3 position;
    vec3 color;
} light;

layout (location = 0) in vec2 tex_coord;
layout (location = 1) in vec2 normal;
layout (location = 2) in vec3 world_position;
layout (location = 3) in vec3 position;
layout (location = 4) in vec3 view_position;


layout (location = 0) out vec4 fragColor;

// We don't need (or want) much ambient light, so 0.1 is fine
const float ambient_strength = 0.1;



void main() {
    vec4 texture = texture(texture_diffuse, tex_coord) * material.base_color;

    if (material.alpha_mode == 2) { // ALPHA MASK
		if (texture.a < material.alpha_cutoff) {
			discard;
		}
	} else if (material.alpha_mode == 3) { // BLEND
        // todo
    }

    // Calculate the ambient color
    // vec3 ambient_color = light.color * ambient_strength;

    // // Calculate the light direction
    // vec3 light_dir = normalize(light.position - world_position);

    // // Calculate the view direction
    // vec3 view_dir = normalize(view_position.xyz - world_position);

    // // Calculate the half vector
    // vec3 half_dir = normalize(view_dir + light_dir);

    // // Calculate the diffuse color
    // float diffuse_strength = max(dot(world_position, light_dir), 0.0);
    // vec3 diffuse_color = light.color * diffuse_strength;

    // // Calculate the specular color
    // float specular_strength = pow(max(dot(world_position, half_dir), 0.0), 32.0);
    // vec3 specular_color = specular_strength * light.color;

    // // Calculate the final color
    // vec3 result = (ambient_color + diffuse_color + specular_color) * texture.xyz;

    fragColor = texture; // vec4(result, 1.0);
}