# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
# endif

uniform vec4 u_color;
uniform sampler2D u_texture;
uniform vec4 u_clip;

varying vec2 v_texcoord;
varying vec2 v_position;

void main() {
    if (v_position.x < u_clip.x || v_position.x > u_clip.x + u_clip.z ||
        v_position.y < u_clip.y || v_position.y > u_clip.y + u_clip.w) {
        discard;
    }
    float alpha = texture2D(u_texture, v_texcoord).a;
    gl_FragColor = vec4(u_color.rgb * alpha, alpha);
}
