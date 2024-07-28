#version 450

flat in int vert_tex_index;
in vec3 vert_normal;
in vec3 vert_position;
in vec3 camera_direction;
in vec2 vert_tex_coord;

out vec4 frag_color;

// Textures later
// uniform sampler2D textures[32];

layout(binding = 1) uniform LightProperties {
    vec3 position;
    vec3 color;
} light_properties;

vec3 half_vec(vec3 v1, vec3 v2) {
    return normalize(v1 + v2);
}

void main() {
    vec3 MATERIAL_COLOR = vec3(0.7, 1.0, 0.6);

    // Params
    float shininess = 42.0;
    vec3 position = vert_position;
    vec3 normal = normalize(vert_normal);
    vec3 light_dir = normalize(vert_position - light_properties.position);
    vec3 camera_dir = normalize(camera_direction);

    // Lambertian Diffuse
    float incident_angle = max(dot(light_dir, normal), 0.0);
    vec3 lambertian_reflectance = light_properties.color * incident_angle;
    vec3 diffuse = lambertian_reflectance * MATERIAL_COLOR;

    // Specular Highlight
    vec3 half_vector = half_vec(camera_dir, light_dir);
    float specular_intensity = pow(max(dot(normal, half_vector), 0.0), shininess);

    vec3 final_color = diffuse + specular_intensity;

    frag_color = vec4(final_color, 1.0);
}