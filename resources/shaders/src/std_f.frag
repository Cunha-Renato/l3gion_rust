#version 450

in vec2 vert_tex_coord;

out vec4 frag_color;

uniform sampler2D texture0;

void main() {
    // frag_color = vec4(vert_tex_coord, 0.0, 1.0);
    frag_color = texture2D(texture0, vert_tex_coord);
}