#include "data/shaders/functions/projections.glsl"

output vec4 position,position_pd;
output float depth;
output vec2 tex_map;

int b1,b2,b3;
vec3 bone_weights;

uniform mat4 camera_inverted,camera_inverted_pd, projection;
uniform float width,height,zFar,zNear,Angle;
uniform mat3x4 bones[128];
uniform mat3x4 bones_pd[128];

uniform mat4 transform;
uniform mat4 transform_pd;

mat4 model,modeld;
void main(void)
{
    b1 = int(weights.x);
    b2 = int(weights.y);
    b3 = int(weights.z);
    bone_weights.x = fract(weights.x);
    bone_weights.y = fract(weights.y);
    bone_weights.z = fract(weights.z);
    
    float weight = bone_weights.z + bone_weights.y + bone_weights.x;
    
    mat4 bone1 = mat4(bones[b1][0],     bones[b1][1],   bones[b1][2],   vec4(0.0,0.0,0.0,1.0));
    mat4 bone2 = mat4(bones[b2][0],     bones[b2][1],   bones[b2][2],   vec4(0.0,0.0,0.0,1.0));
    mat4 bone3 = mat4(bones[b3][0],     bones[b3][1],   bones[b3][2],   vec4(0.0,0.0,0.0,1.0));
    
    mat4 boned1 = mat4(bones_pd[b1][0], bones_pd[b1][1],bones_pd[b1][2],vec4(0.0,0.0,0.0,1.0));
    mat4 boned2 = mat4(bones_pd[b2][0], bones_pd[b2][1],bones_pd[b2][2],vec4(0.0,0.0,0.0,1.0));
    mat4 boned3 = mat4(bones_pd[b3][0], bones_pd[b3][1],bones_pd[b3][2],vec4(0.0,0.0,0.0,1.0));
    
    mat4 skel  = (bone1 *bone_weights.x + bone2 *bone_weights.y + bone3 *bone_weights.z)/weight;
    mat4 skeld = (boned1*bone_weights.x + boned2*bone_weights.y + boned3*bone_weights.z)/weight;
    if (abs(weight)>0.0)
    {
    	model = skel;
    	modeld = skeld;
    }
    else
    {
    	model = transform;
    	modeld = transform_pd;
    }
    
    mat4 modelWiew    = model*camera_inverted;
    mat4 modelWiew_pd = modeld*camera_inverted_pd;
    
    position    = vec4(pos,1)*nextMatrix(modeld,model) * nextMatrix(camera_inverted_pd, camera_inverted);
    position_pd = vec4(pos,1)*modeld * camera_inverted_pd;
    
    vec3 normal = (vec4(bin,0.0) * modelWiew).xyz;
    vec4 motionVector = (position-position_pd) * 0.5;
    float stretch = (dot(motionVector.xyz,normal)+1.0) *0.5;
    gl_Position = mix(position_pd, position, stretch) * projection;
    depth = gl_Position.z;
    position    *= projection;
    position_pd *= projection;
    tex_map = uv;
}