#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
input vec2 tex_map2;
input vec3 normals,viewDir,viewDirTBN;
input vec4 position;

input mat3 TBN;
#if __VERSION__ == 120 || __VERSION__ == 100
#define fragColor gl_FragColor
#else
output vec4 fragColor;
#endif

vec2 velocity_vector;

uniform int lights_count;
uniform vec3 material_diffuse;
uniform vec3 material_specular;
uniform float transparency;
uniform float height_scale;
uniform float reflection_coeff;

uniform sampler2D diffuse_map;
uniform sampler2D light_map;
uniform sampler2D specular_map;
uniform sampler2D normal_map;
uniform samplerCube cubemap;
uniform vec2 normal_map_size;
uniform vec2 texture_displacement;

uniform float material_glow;
uniform bool material_dtex;
uniform bool material_ntex;
uniform bool material_stex;
uniform bool material_htex;
uniform bool material_ltex;
uniform bool material_shadeless;

vec3 normal,lighting;
vec3 specular_factor,specular_mask;
vec4 world_position, color;
uniform mat4 camera_inverted,transform, projection,camera_transform,projection_inverted;

float roughness;
#include "data/shaders/functions/projections.glsl"
#include "data/shaders/lighting/lights.glsl"
vec3 normal_map_from_rg(sampler2D tex,vec2 coords)
{
    vec3 result = texture(tex, coords).rgb*2.0-1.0;
    result.b = sqrt(1.0 - result.r*result.r - result.b*result.b) * 1.5;
    return normalize(result);
}

void main(void)
{
    world_position = position*projection_inverted*camera_transform;
    
    specular_factor = vec3(0.0);
    specular_mask = vec3(0.0);
    normal = normals;//material_htex ? normal_map_from_rg(normal_map, tex_map) * TBN : normals;
    color = material_dtex ? texture(diffuse_map, tex_map) : vec4(material_diffuse,1.0);
    
    vec3 lighting = vec3(0.0);
    float lc = 0.3;
    lighting = material_ltex ? texture(light_map, tex_map2).rgb : vec3(1.0);
    vec3 ambient = textureLod(cubemap, vec3( 0.0, 1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0, 1.0))+lc)/(1.0+lc), 0.0);
    ambient += textureLod(cubemap, vec3( 0.0,-1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0,-1.0))+lc)/(1.0+lc), 0.0);
    
    ambient += textureLod(cubemap, vec3(-1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3(-1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    ambient += textureLod(cubemap, vec3( 1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    
    ambient += textureLod(cubemap, vec3( 0.0, 0.0,-1.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 1.0, 0.0))+lc)/(1.0+lc), 0.0);
    ambient += textureLod(cubemap, vec3( 0.0, 0.0, 1.0), 8.0).rgb * max((dot(normal, vec3( 0.0,-1.0, 0.0))+lc)/(1.0+lc), 0.0);
    roughness = 1.0;
    lighting *= 0.25;
    lighting += sunDiffuse();
    lighting += spotDiffuse();
    lighting += pointDiffuse();
    
    fragColor.rgb = lighting*color.rgb;
    
    fragColor.a = clamp((texture(diffuse_map,tex_map).a-0.5)*5.0+0.5, 0.0, 1.0);
    if (fragColor.a<0.9)
    {
        discard;
    }
}
