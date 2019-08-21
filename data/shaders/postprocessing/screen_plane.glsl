#include "data/shaders/version.glsl"
#define VERTEX
#include "data/shaders/head.h"

output vec2 tex_map;
output vec3 normals,position;
output mat3 TBN;
output vec2 resolution; 
output vec2 pixel_coord;
uniform float width,height;

void main()
{
    resolution = vec2(width,height);
    tex_map = uv;
    normals = nor;
    TBN = mat3(tang,bin,nor);
    pixel_coord = tex_map;
    pixel_coord.x *= width;
    pixel_coord.y *= height;
    gl_Position = vec4(pos, 1.0);
}
