# ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
# else
precision mediump float;
# endif

uniform vec2 u_resolution;
uniform float u_pixelRatio;
uniform float u_alpha;
uniform vec2 u_dimensions;
uniform sampler2D u_texture;
uniform float u_rtt;

uniform vec4 u_radius;
uniform vec4 u_borderWidth;
uniform vec4 u_borderColor;
uniform vec4 u_shadowColor;
uniform vec4 u_shadow;

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

float roundedBox(vec2 p, vec2 s, vec4 r) {
    r.xy = (p.x > 0.0) ? r.yz : r.xw;
    r.x = (p.y > 0.0) ? r.y : r.x;
    vec2 q = abs(p) - s + r.x;
    return (min(max(q.x, q.y), 0.0) + length(max(q, 0.0))) - r.x;
}

float shadowBox(vec2 p, vec2 s, vec4 r) {
    r.xy = (p.x > 0.0) ? r.yz : r.xw;
    r.x = (p.y > 0.0) ? r.y : r.x;
    vec2 q = abs(p) - s + r.x;
    float dist = min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
    return 1.0 - smoothstep(-u_shadow.w, u_shadow.w + u_shadow.z, dist);
}

void main() {
    vec4 color = texture2D(u_texture, v_textureCoords) + v_color;
    vec4 resultColor = vec4(0.0);
    vec2 boxUv = v_nodeCoords.xy * u_dimensions - v_halfDimensions;
    float borderZero = 1.0 - step(0.001, dot(abs(u_borderWidth), vec4(1.0)));
    float edgeWidth = 1.0 / u_pixelRatio;
    float nodeDist = roundedBox(boxUv, v_halfDimensions - edgeWidth, u_radius);
    float nodeAlpha = 1.0 - smoothstep(-0.5 * edgeWidth, 0.5 * edgeWidth, nodeDist);
    float shadowAlpha;

    if (borderZero == 1.0) {
        shadowAlpha = shadowBox(boxUv - u_shadow.xy, v_halfDimensions + u_shadow.w - edgeWidth, u_radius + u_shadow.z);
        resultColor = mix(resultColor, u_shadowColor, shadowAlpha);
        gl_FragColor = mix(resultColor, color, nodeAlpha) * u_alpha;
        return;
    }

    if (v_outerSize.x > v_halfDimensions.x || v_outerSize.y > v_halfDimensions.y) {
        shadowAlpha = shadowBox(boxUv + v_outerBorderUv - u_shadow.xy, v_outerSize + u_shadow.w - edgeWidth, v_outerBorderRadius + u_shadow.z);
    }
    else {
        shadowAlpha = shadowBox(boxUv - u_shadow.xy, v_halfDimensions + u_shadow.w - edgeWidth, u_radius + u_shadow.z);
    }

    float outerDist = roundedBox(boxUv + v_outerBorderUv, v_outerSize - edgeWidth, v_outerBorderRadius);
    float innerDist = roundedBox(boxUv + v_innerBorderUv, v_innerSize - edgeWidth, v_innerBorderRadius);
    float borderDist = max(-innerDist, outerDist);
    float borderAlpha = (1.0 - smoothstep(-0.5 * edgeWidth, 0.5 * edgeWidth, borderDist)) * u_borderColor.a;
    resultColor = mix(resultColor, u_shadowColor, shadowAlpha);
    resultColor = mix(resultColor, color, nodeAlpha);
    resultColor = mix(resultColor, vec4(u_borderColor.rgb, 1.0), borderAlpha);
    gl_FragColor = resultColor * u_alpha;
}
