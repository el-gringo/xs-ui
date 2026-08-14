# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
#endif

attribute vec2 a_position;
attribute vec2 a_textureCoords;
attribute vec4 a_color;
attribute vec4 a_clip;

uniform vec2 u_resolution;

varying vec2 v_texcoord;
varying vec2 v_position;
varying vec4 v_color;
varying vec4 v_clip;

void main() {
    vec2 screenSpace = (a_position / u_resolution * 2.0 - 1.0) * vec2(1, -1);

    gl_Position = vec4(screenSpace, 0.0, 1.0);
    v_texcoord = a_textureCoords;
    v_position = a_position;
    v_color = a_color;
    v_clip = a_clip;
}
