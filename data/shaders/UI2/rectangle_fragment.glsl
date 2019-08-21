#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

uniform sampler2D background;
uniform float width;
uniform float height;
uniform float rect_width;
uniform float rect_height;
uniform vec4  color;
uniform int   use_texture;

input vec2 tex_map;

#if __VERSION__ == 100 || __VERSION__ == 120
  #define fragColor gl_FragColor
#else
    output vec4 fragColor;
#endif

void main()
{
    if (use_texture!=0) {
        fragColor = texture(background, tex_map);
    } else {
        fragColor = color;
    }
}
 
