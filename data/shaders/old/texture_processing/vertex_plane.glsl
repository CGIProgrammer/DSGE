#include "data/shaders/functions/extensions.glsl"

output vec2 tex_map;

void main() {
    tex_map = pos.xy;
    gl_Position = vec4(tex_map*2.0-1.0, 0.0, 1.0);
}
