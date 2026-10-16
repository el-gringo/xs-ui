// One stroke segment per six vertices.
//
// The CPU writes the two endpoints of a segment, a corner selector and the pen's
// half width. Everything else - the direction, the normal, the offset quad, the
// cap extensions - is worked out here, so a stroke costs the CPU two points per
// segment rather than the four corners of a ribbon.
//
// GLSL ES 1.00, like the rest of the renderer and the webviews the older TVs
// ship. The branch-free `mix` in the fragment stage is deliberate: GLSL ES 1.00
// leaves texture sampling in non-uniform control flow undefined, and a select
// costs nothing here.
//
// `highp` is not optional. The endpoints and `v_pixelPos` are absolute screen
// coordinates, up to the framebuffer's own size, and mediump is an fp16 range on
// the hardware this ships to - where one ulp at 2048 is 2.0. A coverage ramp
// that subtracts two such numbers to decide how much of a pixel is ink would come
// out quantised to whole pixels, which is not antialiasing but the opposite of it.
// It costs nothing on a desktop, which is exactly why it would have survived review.
precision highp float;

attribute vec2 a_p0;
attribute vec2 a_p1;
// (t along the segment, s across it). One of (0,-1) (1,-1) (0,1) (1,1).
attribute vec2 a_corner;
attribute float a_halfWidth;
// 0 = round ends, 1 = cut square across (butt), 2 = square, i.e. butt grown by
// half the pen width. A square cap needs no case of its own in the fragment
// stage: it is the butt box with `reach` set.
attribute float a_cap;
attribute vec4 a_color;
attribute vec4 a_clip;

uniform vec2 u_resolution;

// Where this vertex lands, in screen pixels. The clip rectangle arrives in
// screen space, so unlike the quad shader there is nothing to recover here.
varying highp vec2 v_pixelPos;
varying highp vec2 v_p0;
varying highp vec2 v_p1;
// (along, across) in pixels, measured from the middle of the segment.
varying vec2 v_local;
varying highp float v_halfWidth;
varying float v_cap;
varying vec4 v_color;
varying vec4 v_clip;

void main() {
    vec2 axis = a_p1 - a_p0;
    float len = length(axis);
    // A zero-length segment is dropped on the CPU, but the divide still has to
    // be guarded so a degenerate one cannot produce a NaN position that takes
    // the whole batch with it.
    vec2 dir = len > 0.0 ? axis / len : vec2(1.0, 0.0);
    vec2 nrm = vec2(-dir.y, dir.x);

    // A round cap is a half-disc that reaches half the pen width *past* the
    // endpoint, and a square cap reaches exactly that far as well, so the box
    // has to be grown to cover both or the cap is clipped off flat - which is
    // the one thing the distance field is for. A butt cap is the segment itself.
    float reach = (a_cap < 0.5 || a_cap > 1.5) ? a_halfWidth : 0.0;
    vec2 from = a_p0 - dir * reach;
    vec2 to = a_p1 + dir * reach;

    vec2 at = mix(from, to, a_corner.x) + nrm * (a_halfWidth * a_corner.y);

    float ndc_x = at.x / u_resolution.x * 2.0 - 1.0;
    float ndc_y = 1.0 - at.y / u_resolution.y * 2.0;
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);

    // The frame the segment occupies, so the fragment stage can measure a pixel
    // against it without redoing the projection.
    float total = len + reach * 2.0;
    v_local = vec2((a_corner.x - 0.5) * total, a_corner.y * a_halfWidth);
    v_pixelPos = at;
    // Every vertex of the six agrees on these, so they interpolate to a constant.
    v_p0 = a_p0;
    v_p1 = a_p1;
    v_halfWidth = a_halfWidth;
    v_cap = a_cap;
    v_color = a_color;
    v_clip = a_clip;
}
