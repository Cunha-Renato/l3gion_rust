#version 450

in vec2 vert_tex_coord;

out vec4 frag_color;

void main() {
    vec2 _ = vert_tex_coord;
    frag_color = vec4(1.0, 0.0, 0.0, 1.0);
}