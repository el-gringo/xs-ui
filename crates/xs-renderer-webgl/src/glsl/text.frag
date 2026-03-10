precision lowp float;

// Current position
varying vec2 v_position;
varying vec2 texture_coordinate;

uniform vec2 u_viewport;
uniform vec4 u_color;

uniform sampler2D u_texture;

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;

void main() {
    vec2 uv = (v_position - u_rect_min) / (u_rect_max - u_rect_min);
    float alpha = texture2D(u_texture, uv).a;
    gl_FragColor = vec4(u_color.rgb, alpha);
}
