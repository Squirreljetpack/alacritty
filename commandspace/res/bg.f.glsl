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

// Gradient / normal of rounded rect distance field
vec2 normalRoundedRect(vec2 p, vec2 b, float r)
{
    vec2 q = abs(p) - b + r;
    if (q.x > 0.0 && q.y > 0.0)
    {
        return sign(p) * normalize(q);
    }
    if (q.x > q.y)
    {
        return vec2(sign(p.x), 0.0);
    }
    return vec2(0.0, sign(p.y));
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
        float offset = max(0.0, frameOffset);
        vec2 outer_half = half_size - vec2(offset);
        float outer_r = max(2.0, radius - offset);
        float outer_d = sdRoundedRect(p, outer_half, outer_r);

        vec2 inner_half = half_size - vec2(offset + frameThickness);
        float inner_r = max(2.0, radius - (offset + frameThickness));
        float inner_d = sdRoundedRect(p, inner_half, inner_r);

        float outer_mask = 1.0 - smoothstep(-0.5, 0.5, outer_d);
        float inner_mask = smoothstep(-0.5, 0.5, inner_d);
        float stroke_factor = outer_mask * inner_mask;

        if (stroke_factor > 0.0)
        {
            float light_factor = 1.0;
            if (directionalLighting > 0.5)
            {
                vec2 n = normalRoundedRect(p, outer_half, outer_r);
                light_factor = clamp(1.0 + n.y * 0.18, 0.70, 1.30);
            }

            float frame_intensity = clamp(frameColor.a * light_factor, 0.0, 1.0);
            float effective_alpha = stroke_factor * frame_intensity;

            color.rgb = mix(color.rgb, frameColor.rgb, effective_alpha);
            color.a = clamp(color.a + (1.0 - color.a) * effective_alpha, 0.0, 1.0);

            // Inner edge definition for thick frames
            if (frameThickness >= 4.0)
            {
                float inner_rim = 1.0 - smoothstep(0.0, 1.2, abs(inner_d));
                float rim_light = (directionalLighting > 0.5) ? (1.0 - pos.y * 0.25) : 1.0;
                vec3 rim_color = mix(frameColor.rgb, vec3(1.0), 0.15);
                color.rgb = mix(color.rgb, rim_color, inner_rim * 0.18 * rim_light * outer_mask);
            }
        }
    }

    FRAG_COLOR = color;
}
