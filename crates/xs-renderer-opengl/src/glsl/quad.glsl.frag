precision mediump float;

varying vec2 v_pixelPos;
varying vec2 v_screenPos;
varying vec2 v_textureCoords;
varying vec4 v_box;
varying vec4 v_color;
varying vec4 v_gradientStart;
varying vec4 v_gradientEnd;
varying float v_gradientAngle;
varying float v_colorAlpha;
varying vec4 v_shadow;
varying vec4 v_shadowColor;
varying vec4 v_borderColor;
varying vec4 v_borderWidth;
varying vec4 v_borderRadius;
varying vec4 v_clip;

uniform sampler2D u_texture;

// Signed distance of a rounded box centered at the origin.
float sdRoundBox(vec2 p, vec2 b, vec4 r) {
    vec2 q = abs(p) - b + r.xy;
    float min_radius = min(min(r.x, r.y), min(r.z, r.w));
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - min_radius;
}

// Anti-aliased coverage of a rounded box spanning [lower, upper].
float boxAlpha(vec2 lower, vec2 upper, vec2 point, vec4 radius) {
    vec2 center = (lower + upper) * 0.5;
    vec2 half_size = (upper - lower) * 0.5;
    float d = sdRoundBox(point - center, half_size, radius);
    return 1.0 - smoothstep(-1.0, 1.0, d);
}

float erf(float x) {
    float s = sign(x);
    float a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

void main() {
    vec4 texColor = texture2D(u_texture, v_textureCoords);

    // Local, untransformed pixels: keeping the corner radius, border and shadow in
    // the element's own space is what makes them turn with it instead of being
    // squashed against an axis-aligned bounding box.
    vec2 vertex = v_pixelPos;
    vec2 p = v_box.xy;
    vec2 size = v_box.zw;

    vec2 boxL = p;
    vec2 boxU = p + size;

    vec2 contentL = boxL + vec2(v_borderWidth.w, v_borderWidth.z);
    vec2 contentU = boxU - vec2(v_borderWidth.y, v_borderWidth.x);

    float spread = v_shadow.w;
    vec2 shadowL = boxL + v_shadow.xy - spread;
    vec2 shadowU = boxU + v_shadow.xy + spread;

    float border_total = v_borderWidth.x + v_borderWidth.y + v_borderWidth.z + v_borderWidth.w;
    bool has_border = border_total > 0.5;
    bool has_shadow = v_shadow.z > 0.0;

    float box = boxAlpha(contentL, contentU, vertex, v_borderRadius);

    float border = 0.0;
    if (has_border) {
        border = boxAlpha(boxL, boxU, vertex, v_borderRadius)
            - boxAlpha(contentL, contentU, vertex, v_borderRadius);
    }

    float shadow = 0.0;
    if (has_shadow) {
        vec2 shadow_center = (shadowL + shadowU) * 0.5;
        vec2 shadow_half = (shadowU - shadowL) * 0.5;
        float d = sdRoundBox(vertex - shadow_center, shadow_half, v_borderRadius);
        float sigma = v_shadow.z + 0.25;
        // Box-blur edge profile (erf of the distance), evaluated once.
        shadow = 0.5 - 0.5 * erf(d / (sigma * 1.41421356237));
    }

    vec4 fill_color = v_color;
    if (v_gradientAngle >= 0.0) {
        vec2 dir = vec2(cos(v_gradientAngle), sin(v_gradientAngle));
        vec2 rel_pos = vertex - v_box.xy;
        float proj_bl = dot(vec2(0.0, 0.0), dir);
        float proj_br = dot(vec2(size.x, 0.0), dir);
        float proj_tl = dot(vec2(0.0, size.y), dir);
        float proj_tr = dot(size, dir);
        float min_t = min(min(proj_bl, proj_br), min(proj_tl, proj_tr));
        float max_t = max(max(proj_bl, proj_br), max(proj_tl, proj_tr));
        float t = (dot(rel_pos, dir) - min_t) / (max_t - min_t);
        fill_color = mix(v_gradientStart, v_gradientEnd, clamp(t, 0.0, 1.0));
    }

    vec4 content_color = mix(fill_color, texColor, v_colorAlpha * texColor.a);
    vec4 color = mix(vec4(0.0), v_shadowColor, shadow);
    color = mix(color, v_borderColor, border);
    color = mix(color, content_color, box);

    vec2 clipL = v_clip.xy;
    vec2 clipU = v_clip.xy + v_clip.zw;
    if (v_screenPos.x < clipL.x || v_screenPos.x > clipU.x ||
            v_screenPos.y < clipL.y || v_screenPos.y > clipU.y) {
        discard;
    }

    gl_FragColor = color;
}
