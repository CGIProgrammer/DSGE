#include "data/shaders/functions/extensions.glsl"

output vec2 texture_uv;
output vec2 light_uv;
output vec4 position;
output vec3 view_vector;
output mat3 TBN;
output vec2 tex_map;

uniform mat4 vCameraProjection;
uniform mat4 vCameraProjectionInv;
uniform mat4 vCameraTransform;
uniform mat4 vObjectTransform;
mat4 modelView, model;

#ifdef SKELETON
uniform mat3x4 bones[128];
int b1,b2,b3;
vec3 bone_weights;

mat4 model_matrix()
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
        return skel;
    }
    else
    {
        return vObjectTransform;
    }
}
#else
#define model_matrix() vObjectTransform
#endif

void main(void)
{
    model = model_matrix();
    texture_uv  = vec2(uv.x, 1.0-uv.y);
    light_uv = vec2(uv2.x,1.0-uv2.y);

    TBN = transpose(mat3(tang, bin, nor)) * 
        mat3(model[0].xyz, model[1].xyz, model[2].xyz);
    
    position = vec4(pos, 1.0)*model;
    gl_Position = position*vCameraTransform;
    view_vector = gl_Position.xyz;
    gl_Position *= vCameraProjection;
    position.w = gl_Position.w;
    tex_map = gl_Position.xy / gl_Position.w;
}