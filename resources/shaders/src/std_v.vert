#version 450

in vec3 position;
in vec2 tex_coord;

out vec2 vert_tex_coord;

void main() {
    vert_tex_coord = tex_coord;
    gl_Position = vec4(position, 1.0);
}