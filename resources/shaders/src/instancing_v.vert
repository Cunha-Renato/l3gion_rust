#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 instance_position;
layout(location = 3) in vec3 instance_scale;
layout(location = 4) in vec3 instance_rotation_axis;
layout(location = 5) in float instance_rotation_angle;

layout(binding = 0) uniform ViewModel {
    mat4 view;
    mat4 proj;
} view_model;

out vec2 vert_tex_coord;

// Function to construct a rotation matrix given an axis and an angle
mat4 rotation_matrix(vec3 axis, float angle) {
    float cos_angle = cos(angle);
    float sin_angle = sin(angle);
    float one_minus_cos = 1.0 - cos_angle;

    axis = normalize(axis);

    return mat4(
        vec4(cos_angle + axis.x * axis.x * one_minus_cos,         axis.x * axis.y * one_minus_cos - axis.z * sin_angle, axis.x * axis.z * one_minus_cos + axis.y * sin_angle, 0.0),
        vec4(axis.y * axis.x * one_minus_cos + axis.z * sin_angle, cos_angle + axis.y * axis.y * one_minus_cos,         axis.y * axis.z * one_minus_cos - axis.x * sin_angle, 0.0),
        vec4(axis.z * axis.x * one_minus_cos - axis.y * sin_angle, axis.z * axis.y * one_minus_cos + axis.x * sin_angle, cos_angle + axis.z * axis.z * one_minus_cos,         0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

// Function to construct the model matrix
mat4 construct_model_matrix(vec3 translation, vec3 scale, vec3 rotation_axis, float rotation_angle) {
    // Scale matrix
    mat4 scale_matrix = mat4(
        vec4(scale.x, 0.0, 0.0, 0.0),
        vec4(0.0, scale.y, 0.0, 0.0),
        vec4(0.0, 0.0, scale.z, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );

    // Rotation matrix
    mat4 rot_matrix = rotation_matrix(rotation_axis, rotation_angle);

    // Translation matrix
    mat4 translation_matrix = mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(translation, 1.0)
    );

    // Combine all transformations: translate * rotate * scale
    return translation_matrix * rot_matrix * scale_matrix;
}

void main() {
    vert_tex_coord = tex_coord;

    mat4 model = construct_model_matrix(instance_position, instance_scale, instance_rotation_axis, instance_rotation_angle);
    
    gl_Position = view_model.proj * view_model.view * model * vec4(position, 1.0);
}