#version 300 es
precision lowp float;

in vec2 v_position;
in vec2 v_texture_coordinate;

uniform highp vec2 u_rect_min;
uniform highp vec2 u_rect_max;
uniform highp vec4 u_color;
uniform sampler2D u_texture;

out vec4 fragColor;

void main() {
    vec2 uv = v_texture_coordinate;
    float alpha = texture(u_texture, uv).a;
    fragColor = u_color * alpha;
}
