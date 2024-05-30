#version 450

in vec3 position;
in vec2 tex_coord;

layout(binding = 0) uniform ViewModel {
    mat4 view;
    mat4 proj;
} view_model;

out vec2 vert_tex_coord;

void main() {
    vert_tex_coord = tex_coord;
    gl_Position = view_model.proj * view_model.view * vec4(position, 1.0);
}