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

    // Anti-aliasing: smoothstep over 1 pixel
    float alpha = 1.0 - smoothstep(-0.5, 0.5, d);
    vec4 color = vec4(bgColor.rgb, bgColor.a * alpha);

    // Draw frame only if thickness > 0
    if (frameThickness > 0.0)
    {
        // frameOffset: how much to inset the frame from the edge
        float frame_d = d + frameOffset;
        
        // Stroke distance: abs(distance to boundary) - half_thickness
        // Centers the stroke at frameOffset pixels from the edge
        float stroke_d = abs(frame_d + frameThickness * 0.5) - frameThickness * 0.5;
        
        float frame_aa = 1.0 - smoothstep(-0.5, 0.5, stroke_d);
        float factor = frame_aa * frameColor.a;
        
        // Blend frame on top of background
        color.rgb = mix(color.rgb, frameColor.rgb, factor);
        color.a = max(color.a, factor);
    }

    FRAG_COLOR = color;
}
