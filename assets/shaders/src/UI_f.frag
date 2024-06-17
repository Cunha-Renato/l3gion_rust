#version 450

in vec4 inst_color;

out vec4 frag_color;

void main() {
    frag_color = inst_color;
}