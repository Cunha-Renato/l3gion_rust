#version 450

layout(set = 0, binding = 0) uniform ViewProjUBO {
    mat4 view;
    mat4 proj;
} u_ViewProjection;
layout(set = 2, binding = 0) uniform ModelUBO  {
    mat4 data;
} u_Model;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    fragColor = inColor;
    fragTexCoord = inTexCoord;
    gl_Position = u_ViewProjection.view * u_ViewProjection.proj * u_Model.data * vec4(inPosition, 1.0);
}