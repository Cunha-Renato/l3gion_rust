#version 450

#define DEPTH_ARRAY_SCALE 10000

in vec4 vert_id;

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
    if (distance < 1) 
    {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    }
    else {
        // ssbo.data = uvec4(fragCoord.x, data.mouse_position.x, fragCoord.y, data.mouse_position.y);
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
} 