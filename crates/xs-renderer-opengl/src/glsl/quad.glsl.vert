precision mediump float;

attribute vec2 a_position;
attribute vec2 a_textureCoords;
attribute vec4 a_box;
attribute vec4 a_color;
attribute vec4 a_gradientStart;
attribute vec4 a_gradientEnd;
attribute float a_gradientAngle;
attribute float a_colorAlpha;
attribute vec4 a_shadow;
attribute vec4 a_shadowColor;
attribute vec4 a_borderColor;
attribute vec4 a_borderWidth;
attribute vec4 a_borderRadius;
attribute vec4 a_clip;

uniform vec2 u_resolution;

// The element's own untransformed pixel space, which is what the rounded corner,
// border and shadow fields are defined in. Recovered from the box and the texture
// coordinates rather than from `a_position`, because `a_position` has already been
// moved onto the screen by the node's transform. Every vertex of the quad agrees
// on the value, so this interpolates to a constant with no error in it.
varying vec2 v_pixelPos;
// The on-screen position, i.e. after the node's transform. This is what the clip
// test needs, since the clip rectangle arrives in screen space.
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

void main() {
    float ndc_x = a_position.x / u_resolution.x * 2.0 - 1.0;
    float ndc_y = 1.0 - a_position.y / u_resolution.y * 2.0;
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);
    v_pixelPos = a_box.xy + a_textureCoords * a_box.zw;
    v_screenPos = a_position;
    v_textureCoords = a_textureCoords;
    v_box = a_box;
    v_color = a_color;
    v_gradientStart = a_gradientStart;
    v_gradientEnd = a_gradientEnd;
    v_gradientAngle = a_gradientAngle;
    v_colorAlpha = a_colorAlpha;
    v_shadow = a_shadow;
    v_shadowColor = a_shadowColor;
    v_borderColor = a_borderColor;
    v_borderWidth = a_borderWidth;
    v_borderRadius = a_borderRadius;
    v_clip = a_clip;
}
