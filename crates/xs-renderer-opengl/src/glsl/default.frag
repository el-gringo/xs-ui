#version 300 es
precision lowp float;

in vec2 v_position;
in vec2 v_localPosition;
in vec2 v_texture_coordinate;

uniform highp vec2 u_rect_min;
uniform highp vec2 u_rect_max;
uniform highp vec4 u_color;
uniform bool u_texture_enabled;
uniform sampler2D u_texture;

uniform highp vec4 u_borderWidth;
uniform highp vec4 u_borderColor;

out vec4 fragColor;

void main() {
    // Border SDF - normalize by actual dimensions per axis
    vec2 dim = u_rect_max - u_rect_min;
    
    // Normalize border widths by actual dimension per axis
    float top = u_borderWidth.x / dim.y;
    float bottom = u_borderWidth.z / dim.y;
    float left = u_borderWidth.w / dim.x;
    float right = u_borderWidth.y / dim.x;

    float borderDist = min(
        min(v_localPosition.y - top, 1.0 - v_localPosition.y - bottom),
        min(v_localPosition.x - left, 1.0 - v_localPosition.x - right)
    );
    
    if (borderDist < 0.0) {
        fragColor = u_borderColor;
        return;
    }

    // Content
    vec4 content = u_color;
    if (u_texture_enabled) {
        vec2 uv = v_localPosition;
        vec4 texColor = texture(u_texture, uv);
        content = mix(u_color, texColor, texColor.a);
    }
    
    fragColor = content;
}
