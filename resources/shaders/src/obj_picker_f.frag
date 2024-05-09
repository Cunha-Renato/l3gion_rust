#version 450

layout(std430, binding = 2) buffer SSBO {
    vec4 data;
} ssbo;
layout(binding = 0) uniform Data {
    vec2 mouse_position;
    uint uuid;
} data;

out vec4 color;

void main() {
    float distance = distance(gl_FragCoord.xy, data.mouse_position);
    if (distance < 10) 
    {
        color = vec4(0.0, 1.0, 1.0, 1.0);
    }
    else {
        ssbo.data = uvec4(gl_FragCoord.x, data.mouse_position.x, gl_FragCoord.y, data.mouse_position.y);
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
} 