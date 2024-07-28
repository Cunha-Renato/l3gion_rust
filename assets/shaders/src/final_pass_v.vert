#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;

out vec2 vert_tex_coord;

void main() {
    vert_tex_coord = tex_coord;
    
    gl_Position = vec4(position, 1.0);
}