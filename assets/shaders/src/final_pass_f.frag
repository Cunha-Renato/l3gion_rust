#version 450

in vec2 vert_tex_coord;

uniform sampler2D textures;

out vec4 frag_color;

void main() {
    vec2 flipped_tex_coords = vec2(vert_tex_coord.x, vert_tex_coord.y * -1.0);

    frag_color = texture(textures, flipped_tex_coords);
}