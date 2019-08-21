#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/config.h"
#include "data/shaders/functions/extensions.glsl"


uniform sampler2D diffuse_map;

input vec4 position,position_pd;
input vec2 tex_map;

#if __VERSION__ != 100 && __VERSION__ != 110 && __VERSION__ != 120
output vec4 fragColor;
#else
  #define fragColor gl_FragColor
#endif

uniform int material_dtex;
uniform float transparency;

vec2 velocity_vector;

void main(void)
{
    velocity_vector = position.xy/position.w-position_pd.xy/position_pd.w;
    fragColor.rg = velocity_vector;
    if (texture(diffuse_map,tex_map).a<0.5 && material_dtex!=0)
    {
        discard;
    }
}

