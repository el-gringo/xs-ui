precision mediump float;

varying vec2 v_pixelPos;
varying vec2 v_texcoord;
varying vec4 v_color;
varying vec4 v_clip;

uniform sampler2D u_texture;

void main() {
    vec2 clipL = v_clip.xy;
    vec2 clipU = v_clip.xy + v_clip.zw;
    if (v_pixelPos.x < clipL.x || v_pixelPos.x > clipU.x ||
            v_pixelPos.y < clipL.y || v_pixelPos.y > clipU.y) {
        discard;
    }
    vec4 texColor = texture2D(u_texture, v_texcoord);
    gl_FragColor = mix(v_color, texColor, texColor.a);
}
