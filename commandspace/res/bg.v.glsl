#if defined(GLES2_RENDERER)
attribute vec2 aPos;
varying mediump vec2 pos;
#else
layout(location = 0) in vec2 aPos;
out vec2 pos;
#endif

void main() {
    pos = aPos;
    gl_Position = vec4(aPos, 0.0, 1.0);
}
