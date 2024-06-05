#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;

void main() {
    vec2 _ = tex_coord;
    gl_Position = vec4(position, 1.0);
}