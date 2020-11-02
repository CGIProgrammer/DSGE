#include "data/shaders/functions/extensions.glsl"

uniform sampler2D gTexture;
input  vec2 tex_map;
input  vec3 position;

uniform float width,height;

void main()
{
    fragColor.rgb = texture(gTexture,tex_map).rgb * (0.5-position.z*0.5);
}
