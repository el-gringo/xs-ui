#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

uniform vec4 u_color;
uniform sampler2D u_texture;
uniform float u_size;
uniform float u_distanceRange;

varying vec2 v_texcoord;
varying float v_scaledDistRange;

float median(float r, float g, float b) {
    return clamp(b, min(r, g), max(r, g));
}

void main() {
    vec3 sample = texture2D(u_texture, v_texcoord).rgb;
    float sigDist = v_scaledDistRange * (median(sample.r, sample.g, sample.b) - 0.5);
    float opacity = clamp(0.5 - sigDist, 0.0, 1.0) * u_color.a;
    gl_FragColor = vec4(u_color.r * opacity, u_color.g * opacity, u_color.b * opacity, opacity);
}
