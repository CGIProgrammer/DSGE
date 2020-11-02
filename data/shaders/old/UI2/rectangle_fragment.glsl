#include "data/shaders/functions/extensions.glsl"

uniform sampler2D background;
uniform float width;
uniform float height;
uniform float rect_width;
uniform float rect_height;
uniform vec4  color;
uniform int   use_texture;

input vec2 tex_map;

void main()
{
    if (use_texture!=0) {
        fragColor = texture(background, tex_map);
    } else {
        fragColor = color;
    }
}
 
