precision mediump float;

// Current position
varying vec2 v_position;

uniform vec2 u_viewport;
uniform vec4 u_color;

uniform sampler2D u_texture;

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;

void main() {
    float alpha = texture2D(u_texture, (v_position - u_rect_min) / (u_rect_max - u_rect_min)).a;
    gl_FragColor = vec4(u_color.rgb, alpha);
}
