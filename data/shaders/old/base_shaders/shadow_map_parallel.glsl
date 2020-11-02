#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
input float depth;
input vec4 position;
input vec3 viewDir;
uniform float width,height,zFar,zNear,Angle;

uniform sampler2D diffuse_map;

void main()
{
    if (texture(diffuse_map,tex_map).a<0.5) discard;
    fragColor = vec4(depth,depth*depth,depth,1.0);
}
