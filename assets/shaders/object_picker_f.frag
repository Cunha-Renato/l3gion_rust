#version 450

layout(set = 1, binding = 0) uniform sampler2D texSampler;
layout(set = 3, binding = 0) buffer ShaderStorageBufferObject {
    uint selected_id;
} ssbo;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in uint Id;

layout(location = 0) out vec4 outColor;

void main() {
    ssbo.selected_id = Id;

    // outColor = texture(texSampler, fragTexCoord);
    outColor = vec4(1.0, 0.0, 1.0, 1.0);
}