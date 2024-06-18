#version 450

in vec4 v_window_color;

out vec4 frag_color;

void main() {
    frag_color = v_window_color;
}