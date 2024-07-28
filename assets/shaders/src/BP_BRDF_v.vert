#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;
layout(location = 3) in vec4 row_0;
layout(location = 4) in vec4 row_1;
layout(location = 5) in vec4 row_2;
layout(location = 6) in int tex_index;

out int vert_tex_index;

// BRDF
out vec3 vert_normal;
out vec3 vert_position;
out vec3 camera_direction;

out vec2 vert_tex_coord;

layout(binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
    vec3 dir;
} camera;

void main() {
    mat4 model = mat4(
        vec4(row_0.x, row_1.x, row_2.x, 0),
        vec4(row_0.y, row_1.y, row_2.y, 0),
        vec4(row_0.z, row_1.z, row_2.z, 0),
        vec4(row_0.w, row_1.w, row_2.w, 1)
    );

    vert_tex_index = tex_index;
    vert_tex_coord = tex_coord;

    // BRDF
    camera_direction = camera.dir;
    vert_normal = normalize(mat3(transpose(inverse(model))) * normal);
    
    gl_Position = camera.proj * camera.view * model * vec4(position, 1.0);
    vert_position = gl_Position.xyz;
}