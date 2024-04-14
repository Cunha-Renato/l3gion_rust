#version 450

layout(set = 1, binding = 0) uniform sampler2D texSampler;

layout(location = 0) flat in uint outId;
layout(location = 0) out vec4 id;

void main() {
    id = vec4(float(outId), 0.0, 0.0, 0.0);
}