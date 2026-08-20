precision mediump float;

attribute vec2 a_position;
attribute vec2 a_texcoord;
attribute vec4 a_color;
attribute vec4 a_clip;

uniform vec2 u_resolution;

varying vec2 v_pixelPos;
varying vec2 v_texcoord;
varying vec4 v_color;
varying vec4 v_clip;

void main() {
    float ndc_x = a_position.x / u_resolution.x * 2.0 - 1.0;
    float ndc_y = 1.0 - a_position.y / u_resolution.y * 2.0;
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);
    v_pixelPos = a_position;
    v_texcoord = a_texcoord;
    v_color = a_color;
    v_clip = a_clip;
}
