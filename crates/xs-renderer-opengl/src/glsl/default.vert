#version 300 es

in vec2 a_position;
in vec2 a_texture_coordinate;

uniform highp vec2 u_viewport;
uniform highp vec2 u_rect_min;
uniform highp vec2 u_rect_max;

out highp vec2 v_position;
out highp vec2 v_localPosition;
out highp vec2 v_texture_coordinate;

void main() {
    v_position = a_position;
    v_texture_coordinate = a_texture_coordinate;
    v_localPosition = (a_position - u_rect_min) / (u_rect_max - u_rect_min);
    
    vec2 zero_to_one = a_position / u_viewport;
    vec2 clip_space = zero_to_one * 2.0 - 1.0;
    gl_Position = vec4(clip_space.x, -clip_space.y, 0.0, 1.0);
}
