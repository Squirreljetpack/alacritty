#if defined(GLES2_RENDERER)
precision mediump float;
varying vec2 pos;
#define FRAG_COLOR gl_FragColor
#else
in vec2 pos;
out vec4 fragColor;
#define FRAG_COLOR fragColor
#endif

uniform vec2 resolution;
uniform float radius;
uniform vec4 bgColor;
uniform vec4 frameColor;
uniform float frameOffset;
uniform float frameThickness;
uniform float directionalLighting;

// Signed distance to a rounded rectangle
// p: position relative to center in pixels
// b: half-size of the rectangle in pixels
// r: corner radius in pixels
float sdRoundedRect(vec2 p, vec2 b, float r)
{
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;
}

void main()
{
    // Convert NDC-like pos (-1 to 1) to pixel coordinates relative to center
    vec2 p = pos * resolution * 0.5;
    
    // The rectangle fills the entire window
    vec2 half_size = resolution * 0.5;

    // Background rounded rectangle
    float d = sdRoundedRect(p, half_size, radius);

    // Anti-aliasing of the window boundary
    float window_alpha = 1.0 - smoothstep(-0.5, 0.5, d);
    if (window_alpha <= 0.0)
    {
        FRAG_COLOR = vec4(0.0);
        return;
    }

    vec4 color = vec4(bgColor.rgb, bgColor.a * window_alpha);

    // Draw frame only if thickness > 0
    if (frameThickness > 0.0)
    {
        float light_factor = directionalLighting > 0.5 ? clamp(1.0 + pos.y * 0.20, 0.65, 1.35) : 1.0;
        float frame_intensity = clamp(frameColor.a * light_factor, 0.0, 1.0);
        vec3 stroke_rgb = mix(bgColor.rgb, frameColor.rgb, frame_intensity);

        float stroke_factor = 0.0;
        if (frameOffset <= 0.0)
        {
            float inner_d = -(d + frameThickness);
            stroke_factor = 1.0 - smoothstep(-0.5, 0.5, inner_d);
        }
        else
        {
            float stroke_d = max(d + frameOffset, -(d + frameOffset + frameThickness));
            stroke_factor = 1.0 - smoothstep(-0.5, 0.5, stroke_d);
        }

        color.rgb = mix(bgColor.rgb, stroke_rgb, stroke_factor);
        color.a = max(color.a, window_alpha * stroke_factor * frame_intensity);
    }

    FRAG_COLOR = color;
}
