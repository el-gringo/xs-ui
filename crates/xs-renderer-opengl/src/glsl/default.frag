precision mediump float;

// Current position
varying vec2 v_position;

uniform vec2 u_viewport;
uniform vec4 u_color;
uniform bool u_texture_enabled;
uniform sampler2D u_texture;

struct border_side {
    bool enabled;
    float width;
    vec4 color;
};

uniform border_side u_borders[4];

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;

void main() {
    //vec2 uv = (2.0 * v_position.xy - u_viewport) / u_viewport.xx;

    border_side top_border = u_borders[0];
    border_side bottom_border = u_borders[1];
    border_side left_border = u_borders[2];
    border_side right_border = u_borders[3];

    // Probably need to do something here during resizing on fractional values
    if (top_border.enabled && v_position.y < u_rect_min.y + top_border.width) {
        gl_FragColor = top_border.color;
    } else if (bottom_border.enabled && v_position.y > u_rect_max.y - bottom_border.width) {
        gl_FragColor = bottom_border.color;
    } else if (left_border.enabled && v_position.x < u_rect_min.x + left_border.width) {
        gl_FragColor = left_border.color;
    } else if (right_border.enabled && v_position.x > u_rect_max.x - right_border.width) {
        gl_FragColor = right_border.color;
    } else {
        if (u_texture_enabled) {
            gl_FragColor = texture2D(u_texture, (v_position - u_rect_min) / (u_rect_max - u_rect_min));
        } else {
                gl_FragColor = u_color;
        }
    }
}
