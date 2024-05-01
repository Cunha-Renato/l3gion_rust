#version 450

in vec2 vert_tex_coord;
out vec4 frag_color;

uniform sampler2D texture0;

void main() {
    frag_color = vec4(1.0, 0.0, 0.0, 1.0);
}