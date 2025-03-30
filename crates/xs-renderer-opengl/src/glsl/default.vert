// Position of the vertex
attribute vec2 a_position;

// Pixel dimensions of the canvas
uniform vec2 u_viewport;

varying vec2 v_position;

void main() {
    //v_position = a_position;
    v_position = vec2(0., 0.);
    vec2 zero_to_one = a_position / u_viewport;
    vec2 zero_to_two = zero_to_one * 2.0;
    vec2 clip_space = zero_to_two - 1.0;
    vec2 origin_top_left = vec2(1.0, -1.0) * clip_space;
    gl_Position = vec4(origin_top_left, 0.0, 1.0);
}
