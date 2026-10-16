// Coverage of one stroked segment.
//
// The pen's edge is resolved here, per pixel, as a distance to the segment
// rather than as geometry the CPU expanded into a ribbon. That is what buys the
// joins and the caps for free: each segment is a capsule, and a run of capsules
// overlapping at a corner *is* a round join, because the union of two capsules
// at a shared endpoint is exactly the disc of radius halfWidth that rounds that
// corner off. Nothing between two segments knows the other exists.
//
// GLSL ES 1.00, like the rest of the renderer and the webviews the older TVs
// ship.
//
// `highp` throughout, and repeated per varying rather than inherited from the
// vertex stage: a varying's precision is the one declared in the shader that
// *reads* it. It is also load-bearing - `v_pixelPos`, `v_p0` and `v_p1` are
// absolute screen coordinates, and the coverage ramp needs sub-pixel accuracy on
// the difference between two of them. mediump is an fp16 mantissa on the TVs this
// ships to, so at 1024 one ulp is already a whole pixel.
precision highp float;

varying highp vec2 v_pixelPos;
varying highp vec2 v_p0;
varying highp vec2 v_p1;
// (along, across) in pixels, measured from the middle of the segment.
varying highp vec2 v_local;
varying highp float v_halfWidth;
varying float v_cap;
varying vec4 v_color;
varying vec4 v_clip;

void main() {
    vec2 clipL = v_clip.xy;
    vec2 clipU = v_clip.xy + v_clip.zw;
    if (v_pixelPos.x < clipL.x || v_pixelPos.x > clipU.x ||
            v_pixelPos.y < clipL.y || v_pixelPos.y > clipU.y) {
        discard;
    }

    // Round ends: the distance to the segment, with the projection clamped, which
    // turns each end into a half-disc of the pen's radius.
    vec2 axis = v_p1 - v_p0;
    float len2 = dot(axis, axis);
    vec2 rel = v_pixelPos - v_p0;
    float u = clamp(dot(rel, axis) / max(len2, 1.0e-6), 0.0, 1.0);
    float round_sd = length(rel - axis * u) - v_halfWidth;

    // Square and butt ends: the distance to the rectangle the segment occupies,
    // which for a square cap is the butt box grown by half the pen width.
    float len = distance(v_p0, v_p1);
    float reach = (v_cap < 0.5 || v_cap > 1.5) ? v_halfWidth : 0.0;
    vec2 q = abs(v_local) - vec2(len * 0.5 + reach, v_halfWidth);
    float box_sd = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0);

    // A select rather than a branch: both are cheap, and GLSL ES 1.00 leaves
    // anything non-uniform in control flow undefined.
    float sd = mix(round_sd, box_sd, step(0.5, v_cap));

    // Half a pixel is the right window, and not a fudge. Both fields above are
    // true signed distances - unit gradients - so one pixel of screen is one
    // unit of `sd`, and a pixel sitting exactly on the edge covers exactly half.
    // `clamp(0.5 - sd, 0, 1)` is a one-pixel linear ramp, i.e. exactly
    // `smoothstep(-0.5, 0.5, sd)`, and it is why this needs no derivatives:
    // fwidth would be the wrong tool even where it exists, since it would report
    // the raster's own coarse steps rather than the field's true slope.
    float coverage = clamp(0.5 - sd, 0.0, 1.0);

    // Nothing to draw, and blending a zero here would still cost the blend unit
    // and write the framebuffer.
    if (coverage <= 0.0) {
        discard;
    }

    gl_FragColor = vec4(v_color.rgb, v_color.a * coverage);
}
