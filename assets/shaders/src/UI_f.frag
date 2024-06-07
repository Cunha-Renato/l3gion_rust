#version 450

flat in int vert_tex_index;
in vec2 vert_tex_coord;

uniform sampler2D textures[32];

out vec4 frag_color;

void main() {
    if (vert_tex_index < 0) { 
        frag_color = vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        frag_color = texture2D(textures[vert_tex_index], vert_tex_coord);
    }
}