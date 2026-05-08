precision mediump float;

attribute vec2 a_position;
attribute vec2 a_textureCoords;

varying vec2 v_pixelPos;
varying vec2 v_textureCoords;

uniform vec2 u_resolution;
uniform vec4 u_box;
uniform vec4 u_shadow;

void main() {
    float sigma = u_shadow.z + 0.25;
    float padding = 12.0 * sigma + max(abs(u_shadow.x), abs(u_shadow.y)) + max(0.0, u_shadow.w);
    vec2 vertex = mix(u_box.xy - padding, u_box.xy + u_box.zw + padding, a_position);
    float ndc_x = vertex.x / u_resolution.x * 2.0 - 1.0;
    float ndc_y = 1.0 - vertex.y / u_resolution.y * 2.0;
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);
    v_pixelPos = vertex;
    v_textureCoords = a_textureCoords;
}
