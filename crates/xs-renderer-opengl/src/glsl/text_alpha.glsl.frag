# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
# endif

uniform sampler2D u_texture;

varying vec2 v_texcoord;
varying vec2 v_position;
varying vec4 v_color;
varying vec4 v_clip;

void main() {
    if (v_position.x < v_clip.x || v_position.x > v_clip.x + v_clip.z ||
        v_position.y < v_clip.y || v_position.y > v_clip.y + v_clip.w) {
        discard;
    }
    float alpha = texture2D(u_texture, v_texcoord).a;
    gl_FragColor = vec4(v_color.rgb * alpha, alpha);
}
