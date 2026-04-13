# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
# endif

uniform vec4 u_color;
uniform sampler2D u_texture;

varying vec2 v_texcoord;

void main() {
    float alpha = texture2D(u_texture, v_texcoord).a;
    gl_FragColor = vec4(u_color.rgb * alpha, alpha);
}
