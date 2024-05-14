#version 450

in vec3 position;
in vec2 tex_coord;

void main() {
    vec2 _ = tex_coord;
    gl_Position = vec4(position, 1.0);
}