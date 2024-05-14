#version 450

in vec3 position;
in vec2 tex_coord;

layout(binding = 0) uniform UBO {
    vec4 data;
} ubo;

out vec2 vert_tex_coord;
out vec4 vert_ubo_data;

void main() {
    vert_tex_coord = tex_coord;
    vert_ubo_data = ubo.data;
    gl_Position = vec4(position, 1.0);
}