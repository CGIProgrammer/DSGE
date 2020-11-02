#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
input vec4 position;
input vec3 viewDir;
uniform float width,height,zFar,zNear,Angle;

uniform sampler2D diffuse_map;

void main()
{
    if (texture(diffuse_map,tex_map).a<0.5) discard;
    float d = length(viewDir);
    fragColor = vec4(d,d*d,d,1.0);
}
