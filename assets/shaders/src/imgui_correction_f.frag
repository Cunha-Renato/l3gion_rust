
#version 450

in vec2 vert_tex_coord;

uniform sampler2D textures;

out vec4 frag_color;

void main() {
    vec4 image = texture(textures, vert_tex_coord);
    image.rgb = pow(image.rgb, vec3(2.2));
    
    frag_color = image;
}