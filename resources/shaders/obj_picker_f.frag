#version 450

layout(set = 1, binding = 0, rgba32ui) uniform uimage2D image;

layout(location = 0) flat in uvec4 outId;
layout(location = 0) out vec4 _color;

void main() {
    ivec2 pixelCoords = ivec2(gl_FragCoord.xy);
    imageStore(image, pixelCoords, uvec4(255, 0, 0, 255)); // Red color
    _color = vec4(0.0, 0.0, 0.0, 0.0);
}