#version 450

layout(set = 1, binding = 0) uniform sampler2D texSampler;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in uint outId;

layout(location = 0) out vec4 outColor;

void main() {
    // outColor = texture(texSampler, fragTexCoord);
    float red = float(outId) / 25;
    outColor = vec4(red, 1.0, 1.0, 1.0);
}