#include "data/shaders/functions/extensions.glsl"

output vec2 tex_map,tex_map2;
output vec3 normals;
output vec4 position;
output vec3 viewPos;
output float r,depth;
output mat3 TBN;
output vec3 viewDir,viewDirTBN;

uniform mat4 camera_inverted,transform,camera_transform,projection;
uniform float width,height,zFar,zNear,Angle;
uniform mat3x4 bones[128];
int b1,b2,b3;
vec3 bone_weights;

mat4 model;
void main(void)
{
    viewPos = transpose(camera_inverted)[3].xyz;
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
        model = object_transform;
    }
    
    mat4 modelWiew    = model*mat3x4to4(camera_inverted);
    
    position.xyz = (vec4(pos,1.0)*model).xyz;
    TBN = transpose(mat3(tang,cross(nor, tang),nor))*mat3(model[0].xyz,model[1].xyz,model[2].xyz);
    
    viewDir = position.xyz-transpose(mat3x4to4(camera_transform))[3].xyz;
    viewDirTBN = TBN*viewDir;
    
    tex_map = vec2(uv.x,-uv.y);
    tex_map2 = vec2(uv2.x,-uv2.y);
    normals = (vec4(nor,0.0)*model).xyz;
    
    position = vec4(pos, 1.0)*modelWiew;
    depth = position.z;
    gl_Position = position*mat3x4to4(projection);
    position = gl_Position;

    /*position = vec4(pos, 1.0)*(modelWiew*mat3x4to4(projection));
    gl_Position = position;
    depth = position.z;*/
}
