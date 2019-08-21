#include "data/shaders/version.glsl"
#define VERTEX
#include "data/shaders/head.h"
#include "data/shaders/functions/projections.glsl"

output float depth;

output vec4 position;
output vec4 position_pd;
output vec2 tex_map;

uniform mat4 camera_inverted,camera_inverted_pd,transform,transform_pd,projection;
uniform float width,height,zFar,zNear,Angle;
mat4 cam;

void main(void)
{
    mat4 modelWiew    = interpolate(transform_pd,transform,1.0)*interpolaten(camera_inverted_pd,camera_inverted,1.0);
    mat4 modelWiew_pd = interpolate(transform_pd,transform,0.0)*interpolaten(camera_inverted_pd,camera_inverted,0.0);
    
    position    = vec4(pos,1)*modelWiew;
    position_pd = vec4(pos,1)*modelWiew_pd;
    
    vec3 normal = (vec4(nor,0.0) * modelWiew).xyz;
    vec4 motionVector = position-position_pd;
    float stretch = (dot( motionVector.xyz,normal)+1.0);
    if (transform!=transform_pd)
      gl_Position = mix(position_pd, position, stretch*0.5) * projection;
    else
      gl_Position = position * projection;
    position    *= projection;
    position_pd *= projection;
    tex_map = uv;
}
