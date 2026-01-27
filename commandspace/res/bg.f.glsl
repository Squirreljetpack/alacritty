#if defined(GLES2_RENDERER)
precision mediump float;
varying vec2 pos;
#define FRAG_COLOR gl_FragColor
#else
in vec2 pos;
out vec4 fragColor;
#define FRAG_COLOR fragColor
#endif

uniform float radius;
uniform vec4 bgColor;
uniform vec4 frameColor;
uniform float frameOffset;
uniform float frameThickness;

vec2 center = vec2(0.0, 0.0);
vec2 size   = vec2(0.89, 0.89);

float roundedRectangle(vec2 center, vec2 size, float radius, float thickness)
{
    float d = length(max(abs(pos - center), size) - size) - radius;
    return smoothstep(0.6, 0.3, d / thickness * 5.0);
}
float roundedFrame(vec2 center, vec2 size, float radius, float thickness)
{
    float d = length(max(abs(pos - center), size) - size) - radius;
    return smoothstep(1.0, 0.0, abs(d / thickness) * 5.0);
}

void main()
{
    // Draw background
    float alpha = roundedRectangle(center, size, radius, 0.05);
    vec4 color = vec4(bgColor.rgb, bgColor.a * alpha);

    // Draw frame only if thickness > 0
    if (frameThickness > 0.0)
    {
        vec2 frameSize = size * (1.0 - frameOffset);
        float frame_alpha = roundedFrame(center, frameSize, radius, frameThickness);
        // Blend frame on top of background
        color = mix(color, vec4(frameColor.rgb, frameColor.a), frame_alpha);
    }

    FRAG_COLOR = color;
}
