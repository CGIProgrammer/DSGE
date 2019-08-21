#define VERTEX
#define SKELETON
#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

output vec2 tex_map,tex_map2;
output vec3 normals;
output vec4 position;
output float r,depth;
output mat3 TBN;
output vec3 viewDir,viewDirTBN;

int b1,b2,b3;
output vec3 bone_weights;

uniform mat4 camera_inverted,transform,camera_transform,projection;
uniform float width,height,zFar,zNear,Angle;
uniform mat3x4 bones[128];
uniform int render_projection;

mat4 model;
void main(void)
{
    b1 = int(weights.x);
    b2 = int(weights.y);
    b3 = int(weights.z);
    bone_weights.x = fract(weights.x);
    bone_weights.y = fract(weights.y);
    bone_weights.z = fract(weights.z);
    
    float weight = bone_weights.z + bone_weights.y + bone_weights.x;
    bone_weights /= weight;
    mat4 bone1 = mat4(bones[b1][0],     bones[b1][1],   bones[b1][2],   vec4(0.0,0.0,0.0,1.0));
    mat4 bone2 = mat4(bones[b2][0],     bones[b2][1],   bones[b2][2],   vec4(0.0,0.0,0.0,1.0));
    mat4 bone3 = mat4(bones[b3][0],     bones[b3][1],   bones[b3][2],   vec4(0.0,0.0,0.0,1.0));
    
    mat4 skel  = bone1 *bone_weights.x + bone2 *bone_weights.y + bone3 *bone_weights.z;
    if (abs(weight)>0.0)
    {
        model = skel;
    }
    else
    {
        model = transform;
    }
    
    mat4 modelWiew    = model*camera_inverted;
    
    position.xyz = (vec4(pos,1.0)*model).xyz;
    TBN = transpose(mat3(tang,bin,nor))*mat3(model[0].xyz,model[1].xyz,model[2].xyz);
//     TBN = transpose(TBN);
//     TBN[0].xyz = normalize(TBN[0].xyz);
//     TBN[1].xyz = normalize(TBN[1].xyz);
//     TBN[2].xyz = normalize(TBN[2].xyz);
//     TBN = transpose(TBN);
    
    viewDir = position.xyz-transpose(camera_transform)[3].xyz;
    viewDirTBN = TBN*viewDir;
    
    tex_map = vec2(uv.x,-uv.y);
    tex_map2 = vec2(uv2.x,-uv2.y);
    normals = (vec4(nor,0.0)*model).xyz;
    
    position = vec4(pos, 1.0)*(modelWiew*projection);
    
    gl_Position = position;
    mat4 lMVP;
    depth = gl_Position.z/(zFar-zNear);
}
