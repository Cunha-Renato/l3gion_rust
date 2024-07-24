#version 450

in vec2 vert_tex_coord;

uniform sampler2D textures;

out vec4 frag_color;

void main() {
    vec2 flipped_tex_coords = vec2(1.0 - vert_tex_coord.x, 1.0 - vert_tex_coord.y);

    frag_color = texture(textures, flipped_tex_coords);
}