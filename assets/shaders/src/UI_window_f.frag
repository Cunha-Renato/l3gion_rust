#version 450

in vec4 v_window_color;
in vec4 v_window_title_color;
in vec4 v_window_position_and_size;
flat in float v_window_title_height;

out vec4 frag_color;

bool check_bound(vec2 frag, vec4 pos_size) {
    return frag.x >= pos_size.x
        && frag.y <= pos_size.y
        && frag.x <= (pos_size.x + pos_size.z)
        && frag.y >= (pos_size.y - pos_size.w);
}

void main() {
    vec2 frag_coord = gl_FragCoord.xy;
    vec4 window_title_position_and_size = vec4(
        v_window_position_and_size.xyz, 
        v_window_title_height
    );

    // Title Bar
    if (check_bound(frag_coord, window_title_position_and_size)) {
        frag_color = v_window_title_color;
    } 
    // Window
    else if (check_bound(frag_coord, v_window_position_and_size)) {
        frag_color = v_window_color;
    } 
    else {
        discard;
    }
}