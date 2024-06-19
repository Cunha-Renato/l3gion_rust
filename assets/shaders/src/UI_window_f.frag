#version 450

in vec4 v_window_color;
in vec4 v_window_title_color;
in vec4 v_window_position_and_size;
in vec4 v_window_title_position_and_size;

out vec4 frag_color;

bool check_bound(vec2 frag, vec4 pos_size) {
    return frag.x >= pos_size.x
        && frag.y <= pos_size.y
        && frag.x <= (pos_size.x + pos_size.z)
        && frag.y >= (pos_size.y - pos_size.w);
}

void main() {
    vec2 frag_coord = gl_FragCoord.xy;

    // Window
    if (check_bound(frag_coord, v_window_position_and_size)) {
        frag_color = v_window_color;
    } 
    // Title Bar
    else if (check_bound(frag_coord, v_window_title_position_and_size)) {
        frag_color = v_window_title_color;
    } 
    else {
        discard;
    }
}