precision mediump float;

varying vec2 v_pixelPos;
varying vec2 v_texCoord;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform vec4 u_box;
uniform vec4 u_color;
uniform float u_colorAlpha;
uniform vec4 u_shadow;
uniform vec4 u_shadowColor;
uniform vec4 u_borderColor;
uniform vec4 u_borderWidth;
uniform float u_borderRadius;

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

float roundedBoxShadowX(float x, float y, float sigma, float corner, vec2 halfSize) {
    float delta = min(halfSize.y - corner - abs(y), 0.0);
    float curved = halfSize.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    vec2 integral = 0.5 + 0.5 * erf((x + vec2(-curved, curved)) * (sqrt(0.5) / sigma));
    return integral.y - integral.x;
}

float roundedBoxShadow(vec2 lower, vec2 upper, vec2 point, float sigma, float corner) {
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
        value += roundedBoxShadowX(point.x, point.y - y, sigma, corner, halfSize) * gaussian(y, sigma) * step;
        y += step;
    }

    return value;
}

float roundedBoxSDF(vec2 p, vec2 size, float corner) {
    vec2 d = abs(p) - (size - corner);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - corner;
}

void main() {
    vec2 r = u_resolution;
    vec4 texColor = texture2D(u_texture, v_texCoord);

    vec2 vertex = v_pixelPos.xy;
    vec2 p = u_box.xy;
    vec2 size = u_box.zw;
    float shadowBlur = u_shadow.z;

    vec2 boxL = p;
    vec2 boxU = p + size;

    vec2 borderL = boxL - vec2(u_borderWidth.w, u_borderWidth.z);
    vec2 borderU = boxU + vec2(u_borderWidth.y, u_borderWidth.x);

    vec2 shadowL = borderL + u_shadow.xy;
    vec2 shadowU = borderU + u_shadow.xy;

    float box = roundedBoxShadow(boxL, boxU, vertex, 0.25, u_borderRadius);
    float border = roundedBoxShadow(borderL, borderU, vertex, 0.25, u_borderRadius);
    float shadow = roundedBoxShadow(shadowL, shadowU, vertex, shadowBlur, u_borderRadius);

    vec4 fillColor = mix(u_color, texColor, texColor.a * u_colorAlpha);
    fillColor.a = max(fillColor.a, texColor.a);

    vec4 resetColor = vec4(0.0);
    vec4 color = mix(resetColor, u_shadowColor, shadow);
    color = mix(color, u_borderColor, border);
    color = mix(color, fillColor, box);
    gl_FragColor = color;
}
