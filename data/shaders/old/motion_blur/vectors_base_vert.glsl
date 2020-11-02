#include "data/shaders/functions/projections.glsl"

output float depth;

output vec4 position;
output vec4 position_pd;
output vec2 tex_map;

uniform mat4 camera_inverted,camera_inverted_pd,projection;

#if MAX_INSTANCES==1
uniform mat4 transform;
uniform mat4 transform_pd;
#else
uniform mat4 transform[MAX_INSTANCES];
uniform mat4 transform_pd[MAX_INSTANCES];
#endif

uniform float width,height,zFar,zNear,Angle;
mat4 cam;

void main(void)
{
    mat4 ci = camera_inverted;
    mat4 cip= camera_inverted_pd;

    #if MAX_INSTANCES==1
    mat4 tr = transform;
    mat4 tr_pd = transform_pd;
    #else 
    mat4 tr = transform[gl_InstanceID];
    mat4 tr_pd = transform_pd[gl_InstanceID];
    #endif

    mat4 modelWiew    = nextMatrix(tr_pd, tr) * nextMatrix(cip,ci);
    mat4 modelWiew_pd = tr_pd * cip;

    position    = vec4(pos,1)*modelWiew;
    position_pd = vec4(pos,1)*modelWiew_pd;
    
    vec3 normal = (vec4(bin,0.0) * modelWiew).xyz;
    vec4 motionVector = (position-position_pd) * 0.5;
    float stretch = (dot( motionVector.xyz,normal)+1.0) * 0.5;
    if (position!=position_pd)
      gl_Position = mix(position_pd, position, stretch) * projection;
    else
      gl_Position = position * projection;
    position    *= projection;
    position_pd *= projection;
    tex_map = uv;
}
