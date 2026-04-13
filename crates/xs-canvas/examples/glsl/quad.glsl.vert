# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
# endif

attribute vec2 a_position;
attribute vec2 a_textureCoords;
attribute vec4 a_color;
attribute vec2 a_nodeCoords;

uniform vec2 u_resolution;
uniform float u_pixelRatio;
uniform float u_rtt;
uniform vec2 u_dimensions;

uniform vec4 u_shadow;
uniform vec4 u_radius;
uniform vec4 u_borderWidth;
uniform float u_borderGap;
uniform float u_borderAlign;

varying vec4 v_color;
varying vec2 v_textureCoords;
varying vec2 v_nodeCoords;

varying vec2 v_innerSize;
varying vec2 v_outerSize;
varying vec2 v_outerBorderUv;
varying vec2 v_innerBorderUv;
varying vec4 v_innerBorderRadius;
varying vec4 v_outerBorderRadius;
varying vec2 v_halfDimensions;

void main() {
    vec2 screenSpace = vec2(2.0 / u_resolution.x, -2.0 / u_resolution.y);
    vec2 edge = clamp(a_nodeCoords * 2.0 - vec2(1.0), -1.0, 1.0);
    float borderZero = 1.0 - step(0.001, dot(abs(u_borderWidth), vec4(1.0)));

    vec2 edgeOffset = edge * ((u_shadow.w * 2.0) + u_shadow.z) + u_shadow.xy;
    vec2 vertexPos = (a_position + edge + edgeOffset) * u_pixelRatio;

    v_innerSize = vec2(0.0);
    v_outerSize = vec2(0.0);

    if (borderZero == 0.0) {
        vec4 adjustedBorderWidth = u_borderWidth - 1.0 + clamp(u_borderWidth, -1.0, 1.0);

        float borderTop = adjustedBorderWidth.x;
        float borderRight = adjustedBorderWidth.y;
        float borderBottom = adjustedBorderWidth.z;
        float borderLeft = adjustedBorderWidth.w;

        v_outerBorderUv = vec2(0.0);
        v_innerBorderUv = vec2(0.0);

        vec2 borderSize = vec2(borderRight + borderLeft, borderTop + borderBottom);
        vec2 extraSize = borderSize * u_borderAlign;
        float gapLeft = step(0.001, borderLeft) * u_borderGap;
        float gapRight = step(0.001, borderRight) * u_borderGap;
        float gapTop = step(0.001, borderTop) * u_borderGap;
        float gapBottom = step(0.001, borderBottom) * u_borderGap;
        vec2 gapSize = vec2(gapLeft + gapRight, gapTop + gapBottom);

        v_outerSize = (u_dimensions + gapSize + extraSize) * 0.5;
        v_innerSize = v_outerSize - borderSize * 0.5;

        // Use sign() to avoid branching
        vec2 borderDiff = vec2(borderRight - borderLeft, borderBottom - borderTop);
        vec2 signDiff = sign(borderDiff);
        borderDiff = abs(borderDiff);

        vec2 gapDiff = vec2(gapRight - gapLeft, gapBottom - gapTop);
        vec2 signGapDiff = sign(gapDiff);
        gapDiff = abs(gapDiff);

        v_outerBorderUv = -signDiff * borderDiff * u_borderAlign * 0.5 - signGapDiff * gapDiff * 0.5;
        v_innerBorderUv = v_outerBorderUv + signDiff * borderDiff * 0.5;

        v_outerBorderRadius = vec4(
                max(0.0, u_radius.x + max(borderTop * u_borderAlign + u_borderGap, borderLeft * u_borderAlign + u_borderGap)),
                max(0.0, u_radius.y + max(borderTop * u_borderAlign + u_borderGap, borderRight * u_borderAlign + u_borderGap)),
                max(0.0, u_radius.z + max(borderBottom * u_borderAlign + u_borderGap, borderRight * u_borderAlign + u_borderGap)),
                max(0.0, u_radius.w + max(borderBottom * u_borderAlign + u_borderGap, borderLeft * u_borderAlign + u_borderGap))
            );

        v_innerBorderRadius = vec4(
                max(0.0, v_outerBorderRadius.x - max(borderTop, borderLeft)),
                max(0.0, v_outerBorderRadius.y - max(borderTop, borderRight)),
                max(0.0, v_outerBorderRadius.z - max(borderBottom, borderRight)),
                max(0.0, v_outerBorderRadius.w - max(borderBottom, borderLeft))
            );

        vec2 edgeOffsetExtra = step(u_dimensions * 0.5, v_outerSize) * edge * (extraSize + u_borderGap);
        edgeOffset += edgeOffsetExtra;

        vertexPos = (a_position + edge + edgeOffset) * u_pixelRatio;
    }

    gl_Position = vec4(vertexPos.x * screenSpace.x - 1.0, -sign(screenSpace.y) * (vertexPos.y * -abs(screenSpace.y)) + 1.0, 0.0, 1.0);

    v_halfDimensions = u_dimensions * 0.5;
    v_color = a_color;
    v_nodeCoords = a_nodeCoords + (screenSpace + edgeOffset) / (u_dimensions);
    v_textureCoords = a_textureCoords + (screenSpace + edgeOffset) / (u_dimensions);
}
