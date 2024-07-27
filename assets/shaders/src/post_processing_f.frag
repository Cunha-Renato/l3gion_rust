#version 450

in vec2 vert_tex_coord;

uniform sampler2D textures;

out vec4 frag_color;

void main() {
    vec2 flipped_tex_coords = vec2(1.0 - vert_tex_coord.x, 1.0 - vert_tex_coord.y);

    vec4 tex_color = texture(textures, flipped_tex_coords);
    
    if (tex_color.r == 0.0 && tex_color.g == 0.0 && tex_color.b == 0.0) {
        frag_color = vec4(0.0, 1.0, 0.0, 1.0);
        // frag_color = tex_color;
    } else {
        frag_color = tex_color;
        // frag_color = vec4(0.0, 1.0, 0.0, 1.0);
    }
}