#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
input float depth;

#if __VERSION__ != 100 && __VERSION__ != 110 && __VERSION__ != 120
output vec4 fragColor;
#else
  #define fragColor gl_FragColor
#endif

uniform sampler2D diffuse_map;

void main()
{
    if (texture(diffuse_map,tex_map).a<0.5) discard;
    fragColor.rgb = vec3(1.0);
    //fragColor.a = 1.0;
}
