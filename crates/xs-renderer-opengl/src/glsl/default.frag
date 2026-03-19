#version 300 es
precision lowp float;

in vec2 v_position;
in vec2 v_local_position;
in vec2 v_texture_coordinate;

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;
uniform vec4 u_color;
uniform bool u_texture_enabled;
uniform sampler2D u_texture;

uniform vec4 u_border_width;
uniform vec4 u_border_color;

out vec4 frag_color;

void main() {
    // Border SDF - normalize by actual dimensions per axis
    vec2 dim = u_rect_max - u_rect_min;

    // Normalize border widths by actual dimension per axis
    float top = u_border_width.x / dim.y;
    float bottom = u_border_width.z / dim.y;
    float left = u_border_width.w / dim.x;
    float right = u_border_width.y / dim.x;

    float border_dist = min(
            min(v_local_position.y - top, 1.0 - v_local_position.y - bottom),
            min(v_local_position.x - left, 1.0 - v_local_position.x - right)
        );

    if (border_dist < 0.0) {
        frag_color = u_border_color;
        return;
    }

    // Content
    vec4 content = u_color;
    if (u_texture_enabled) {
        vec2 uv = v_local_position;
        vec4 texture_color = texture(u_texture, uv);
        content = mix(u_color, texture_color, texture_color.a);
    }

    frag_color = content;
}
