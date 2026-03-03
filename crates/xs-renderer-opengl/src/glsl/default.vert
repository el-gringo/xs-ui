precision lowp float;

// Position of the vertex
attribute vec2 a_position;
attribute vec2 a_uv;

// Pixel dimensions of the canvas
uniform vec2 u_viewport;

varying vec2 v_position;
varying vec2 texture_coordinate;

void main() {
    v_position = a_position;
    vec2 zero_to_one = a_position / u_viewport;
    vec2 zero_to_two = zero_to_one * 2.0;
    vec2 clip_space = zero_to_two - 1.0;
    vec2 origin_top_left = vec2(1.0, -1.0) * clip_space;
    texture_coordinate = a_uv;
    gl_Position = vec4(origin_top_left, 0.0, 1.0);
}
