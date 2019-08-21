#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

uniform sampler2D font;
uniform vec4 color;
uniform vec2 char_size;
uniform int char_index;

uniform float width,height;
uniform int char_hl;
uniform vec2 element_size;

input vec2 tex_map;

#if __VERSION__ == 100 || __VERSION__ == 120
  #define fragColor gl_FragColor
#else
    output vec4 fragColor;
#endif

void main()
{
    
    if (char_index>32)
    {
        fragColor = vec4(color.rgb, texture(font, tex_map).r * color.a);
    }
    else
    {
        fragColor = vec4(0.0,0.0,0.0,0.0);
    }
}
 
