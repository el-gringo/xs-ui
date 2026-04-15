precision mediump float;

attribute vec2 a_position;
attribute vec2 a_texCoord;

varying vec2 v_pixelPos;
varying vec2 v_texCoord;

uniform vec2 u_resolution;
uniform vec4 u_box;
uniform vec4 u_shadow;

void main() {
    float padding = 12.0 * u_shadow.z;
    vec2 vertex = mix(u_box.xy - padding, u_box.xy + u_box.zw + padding, a_position);
    float ndc_x = vertex.x / u_resolution.x * 2.0 - 1.0;
    float ndc_y = 1.0 - vertex.y / u_resolution.y * 2.0;
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);
    v_pixelPos = vertex;
    v_texCoord = a_texCoord;
}
