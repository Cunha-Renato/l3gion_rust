#version 450

in vec2 vert_tex_coord;
in vec4 vert_ubo_data;

out vec4 frag_color;

void main() {
    frag_color = vert_ubo_data;
}