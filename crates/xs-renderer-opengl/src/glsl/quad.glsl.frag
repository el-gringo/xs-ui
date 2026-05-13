precision mediump float;

varying vec2 v_pixelPos;
varying vec2 v_textureCoords;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform vec4 u_box;
uniform vec4 u_color;
uniform vec4 u_gradientStart;
uniform vec4 u_gradientEnd;
uniform float u_gradientAngle;
uniform float u_colorAlpha;
uniform vec4 u_shadow;
uniform vec4 u_shadowColor;
uniform vec4 u_borderColor;
uniform vec4 u_borderWidth;
uniform vec4 u_borderRadius;
uniform vec4 u_clip;

vec4 erf(vec4 x) {
    vec4 s = sign(x), a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

vec2 erf(vec2 x) {
    vec2 s = sign(x), a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

float gaussian(float x, float sigma) {
    const float pi = 3.141592653589793;
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * pi) * sigma);
}

float roundedBoxShadowX(float x, float y, float sigma, vec4 radius, vec2 halfSize) {
    float corner = x < 0.0
        ? (y < 0.0 ? radius.x : radius.w)
        : (y < 0.0 ? radius.y : radius.z);
    float delta = min(halfSize.y - corner - abs(y), 0.0);
    float curved = halfSize.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    vec2 integral = 0.5 + 0.5 * erf((x + vec2(-curved, curved)) * (sqrt(0.5) / sigma));
    return integral.y - integral.x;
}

float roundedBoxShadow(vec2 lower, vec2 upper, vec2 point, float sigma, vec4 radius) {
    vec2 center = (lower + upper) * 0.5;
    vec2 halfSize = (upper - lower) * 0.5;
    point -= center;

    float low = point.y - halfSize.y;
    float high = point.y + halfSize.y;
    float start = clamp(-3.0 * sigma, low, high);
    float end = clamp(3.0 * sigma, low, high);

    float step = (end - start) / 4.0;
    float y = start + step * 0.5;
    float value = 0.0;
    for (int i = 0; i < 4; i++) {
        value += roundedBoxShadowX(point.x, point.y - y, sigma, radius, halfSize) * gaussian(y, sigma) * step;
        y += step;
    }

    return value;
}

void main() {
    vec2 r = u_resolution;
    vec4 texColor = texture2D(u_texture, v_textureCoords);

    vec2 vertex = v_pixelPos.xy;
    vec2 p = u_box.xy;
    vec2 size = u_box.zw;
    float shadowBlur = u_shadow.z;

    vec2 boxL = p;
    vec2 boxU = p + size;

    vec2 contentL = boxL + vec2(u_borderWidth.w, u_borderWidth.z);
    vec2 contentU = boxU - vec2(u_borderWidth.y, u_borderWidth.x);

    float spread = u_shadow.w;
    vec2 shadowL = boxL + u_shadow.xy - spread;
    vec2 shadowU = boxU + u_shadow.xy + spread;

    float box = roundedBoxShadow(contentL, contentU, vertex, 0.25, u_borderRadius);
    float border = roundedBoxShadow(boxL, boxU, vertex, 0.25, u_borderRadius);
    float shadow = roundedBoxShadow(shadowL, shadowU, vertex, shadowBlur + 0.25, u_borderRadius);

    vec4 fillColor = u_color;
    if (u_gradientAngle >= 0.0) {
        vec2 dir = vec2(cos(u_gradientAngle), sin(u_gradientAngle));
        vec2 boxSize = u_box.zw;
        vec2 relPos = vertex - u_box.xy;
        float projBL = dot(vec2(0.0, 0.0), dir);
        float projBR = dot(vec2(boxSize.x, 0.0), dir);
        float projTL = dot(vec2(0.0, boxSize.y), dir);
        float projTR = dot(boxSize, dir);
        float minT = min(min(projBL, projBR), min(projTL, projTR));
        float maxT = max(max(projBL, projBR), max(projTL, projTR));
        float t = (dot(relPos, dir) - minT) / (maxT - minT);
        fillColor = mix(u_gradientStart, u_gradientEnd, clamp(t, 0.0, 1.0));
    }

    vec4 resetColor = vec4(0.0);
    vec4 color = mix(resetColor, u_shadowColor, shadow);
    color = mix(color, u_borderColor, border);
    color = mix(color, fillColor, box);

    vec2 clipL = u_clip.xy;
    vec2 clipU = u_clip.xy + u_clip.zw;
    if (v_pixelPos.x < clipL.x || v_pixelPos.x > clipU.x ||
        v_pixelPos.y < clipL.y || v_pixelPos.y > clipU.y) {
        discard;
    }

    gl_FragColor = color;
}
