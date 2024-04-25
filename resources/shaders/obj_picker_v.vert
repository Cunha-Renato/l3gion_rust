#version 450

layout(set = 0, binding = 0) uniform ViewProjUBO {
    mat4 view;
    mat4 proj;
} u_ViewProjection;
layout(set = 2, binding = 0) uniform ModelUBO_DYNAMIC  {
    mat4 data;
    uvec4 id;
} u_Model;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out uvec4 outId;

void main() {
    gl_Position = u_ViewProjection.proj * u_ViewProjection.view * u_Model.data * vec4(inPosition, 1.0);

    outId = u_Model.id;
}