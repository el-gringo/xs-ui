# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
#endif

attribute vec2 a_position;
attribute vec2 a_textureCoords;

uniform vec2 u_resolution;
uniform mat3 u_transform;
uniform float u_pixelRatio;

varying vec2 v_texcoord;

void main() {
    vec2 scrolledPosition = a_position;
    vec2 transformedPosition = (u_transform * vec3(scrolledPosition, 1)).xy;

    // Calculate screen space with pixel ratio
    vec2 screenSpace = (transformedPosition * u_pixelRatio / u_resolution * 2.0 - 1.0) * vec2(1, -1);

    gl_Position = vec4(screenSpace, 0.0, 1.0);
    v_texcoord = a_textureCoords;
}
