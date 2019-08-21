#include "data/shaders/version.glsl"
#define VERTEX

#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

output vec2 tex_map;
output vec2 tex_map2;
output vec3 normals;
output float r,depth;
output mat3 TBN;
output vec3 viewDirTBN,viewDir;

output vec4 position;
uniform int render_projection;

uniform mat4 camera_inverted,transform, projection,camera_transform;
uniform float width,height,zFar,zNear,Angle;


float facosf(float x)
{
    return (-0.69813170079773212 * x * x - 0.87266462599716477) * x + 1.5707963267948966;
}

void main(void)
{
    mat4 model = transform;
    vec3 viewPos = transpose(camera_inverted)[3].xyz;
    mat4 modelWiew = model*camera_inverted;
    TBN = transpose(mat3(tang,bin,nor))*mat3(model[0].xyz,model[1].xyz,model[2].xyz);
    
    tex_map  = vec2(uv.x, 1.0-uv.y);
    tex_map2 = vec2(uv2.x,1.0-uv2.y);
    normals = (vec4(nor,0.0)*model).xyz;
    position.xyz = (vec4(pos,1.0)*model).xyz;
    
    viewDir = position.xyz-transpose(camera_transform)[3].xyz;
    viewDirTBN = TBN*viewDir;
    
    position = vec4(pos, 1.0)*(modelWiew*projection);
    gl_Position = position;
    depth = gl_Position.z/(zFar-zNear);
}
