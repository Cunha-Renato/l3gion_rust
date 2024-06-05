#version 450

layout(std430, binding = 2) buffer SSBO {
    uvec4 data;
} ssbo;
layout(binding = 0) uniform Data {
    vec2 mouse_position;
    uint uuid;
} data;

out vec4 color;

void main() {
    float distance = distance(gl_FragCoord.xy, data.mouse_position);
    float id_color = float(data.uuid) / 255.0;

    color = vec4(id_color, 0.0, 0.2, 1.0);
    if (distance < 10) 
    {
        color = vec4(id_color, id_color, id_color, 1.0);
        ssbo.data = uvec4(data.uuid, 0, 0, 0);
    }
} 