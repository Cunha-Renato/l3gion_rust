#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 2) in vec4 window_color;
layout(location = 3) in vec4 window_title_color;
layout(location = 4) in vec4 window_position_and_size;
layout(location = 5) in float window_title_height;

out vec4 v_window_color;
out vec4 v_window_title_color;
out vec4 v_window_position_and_size;
out float v_window_title_height;

void main() {
    vec2 _ = tex_coord;
    
    v_window_color = window_color;
    v_window_title_color = window_title_color;
    v_window_position_and_size = window_position_and_size; 
    v_window_title_height = window_title_height;
    gl_Position = vec4(position, 1.0);
}