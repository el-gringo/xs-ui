#version 300 es
precision lowp float;

in vec2 v_position;
in vec2 v_texture_coordinate;

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;
uniform vec4 u_color;
uniform vec2 u_viewport;
uniform sampler2D u_texture;

out vec4 fragColor;

void main() {
    float alpha = texture(u_texture, v_texture_coordinate).a;
    fragColor = u_color * alpha;
}
