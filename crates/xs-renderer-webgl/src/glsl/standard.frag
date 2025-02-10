precision mediump float;

// Color of the rect
uniform vec4 u_color;

struct border_side {
    bool enabled;
    float width;
    vec4 color;
};

uniform border_side u_borders[4];

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;

// Current position
varying vec2 v_position;

void main() {
    border_side top_border = u_borders[0];
    border_side bottom_border = u_borders[1];
    border_side left_border = u_borders[2];
    border_side right_border = u_borders[3];

    // Probably need to do something here during resizing on fractional values
    if (top_border.enabled && v_position.y < u_rect_min.y + top_border.width) {
        gl_FragColor = top_border.color;
        return;
    } else if (bottom_border.enabled && v_position.y > u_rect_max.y - bottom_border.width) {
        gl_FragColor = bottom_border.color;
        return;
    } else if (left_border.enabled && v_position.x < u_rect_min.x + left_border.width) {
        gl_FragColor = left_border.color;
        return;
    } else if (right_border.enabled && v_position.x > u_rect_max.x - right_border.width) {
        gl_FragColor = right_border.color;
        return;
    }
    gl_FragColor = u_color;
}
