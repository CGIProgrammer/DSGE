#version 420 core
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

in vec2 tex_map;
in vec3 normals,viewDir;
in vec4 position;
in float depth;

//out vec4 fragColor[8];

vec4 fragColor;

uniform int lights_count;
uniform vec3 material_diffuse;
uniform vec3 material_specular;
uniform float transparency;
uniform float height_scale;
uniform float reflection_coeff;

uniform sampler2D diffuse_map;
uniform sampler2D specular_map;
uniform samplerCube cubemap;
uniform vec2 texture_displacement;

uniform float material_glow;
uniform bool material_dtex;
uniform bool material_stex;

vec3 normal,lighting;
vec3 specular_factor,specular_mask;
vec4 world_position, color;
uniform mat4 camera_inverted,transform, projection,camera_transform,projection_inverted;

layout(RGBA16F) uniform coherent volatile  image3D VoxelStoreLoD[6];

#include "data/shaders/functions/projections.glsl"
#include "data/shaders/lighting/lights.glsl"

void main(void)
{
    world_position = position*projection_inverted*camera_transform;
    
    specular_factor = vec3(0.0);
    specular_mask = vec3(0.0);
    normal = normals;
    color = material_dtex ? texture(diffuse_map, tex_map) : vec4(material_diffuse,1.0);
    
    lighting = vec3(material_glow);
    lighting += sunDiffuse();
    lighting += spotDiffuse();
    color.xyz *= lighting;
    
    
    world_position.xyz -= transpose(camera_transform)[3].xyz;
    world_position *= projection;
    //world_position*= 2.0;
    world_position = world_position*0.5+0.5;
    //world_position*= 256;
    for (int lod=0;lod<6;lod++)
    {
        int size = 256>>lod;
        ivec3 vox_pos = ivec3(world_position.xyz*size);
        if (vox_pos.x>=0 && vox_pos.x<size &&
            vox_pos.y>=0 && vox_pos.y<size &&
            vox_pos.z>=0 && vox_pos.z<size)
        {
            if (abs((vec4(normal,0.0)*camera_transform).z)>=0.707)
            {
                vec4 val = imageLoad(VoxelStoreLoD[lod], vox_pos);
                imageStore(VoxelStoreLoD[lod], vox_pos, val + color);
            }
        }
    }
    //imageStore(voxel_image, ivec3((position.xy/position.w*0.5+0.5)*255,0), color);
    
    float alpha = clamp((texture(diffuse_map,tex_map).a-0.35)*5.0,0.0,1.0);
    if (alpha<0.5)
    {
        discard;
        return;
    }
}
