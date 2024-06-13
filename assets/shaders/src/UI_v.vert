#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 2) in vec4 color;
layout(location = 3) in vec4 row_0;
layout(location = 4) in vec4 row_1;
layout(location = 5) in vec4 row_2;

out vec4 inst_color;

void main() {
    vec2 _ = tex_coord;
    mat4 model = mat4(
        vec4(row_0.x, row_1.x, row_2.x, 0),
        vec4(row_0.y, row_1.y, row_2.y, 0),
        vec4(row_0.z, row_1.z, row_2.z, 0),
        vec4(row_0.w, row_1.w, row_2.w, 1)
    );
    
    inst_color = color;
    gl_Position = model * vec4(position, 1.0);
}