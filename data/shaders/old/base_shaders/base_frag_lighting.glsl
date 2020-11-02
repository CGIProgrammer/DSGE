#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
input vec2 tex_map2;
input vec3 normals,viewDir,viewDirTBN;
input vec4 position;

input mat3 TBN;

vec2 velocity_vector;

uniform int lights_count;
uniform vec3 material_diffuse;
uniform vec3 material_specular;
uniform float material_fresnel;
uniform float material_metallic;
uniform float material_roughness;
uniform float material_glow;

uniform float transparency;
uniform float height_scale;
uniform float reflection_coeff;

uniform sampler2D diffuse_map;
uniform sampler2D light_map;
uniform sampler2D specular_map;
uniform sampler2D roughness_map;
uniform sampler2D normal_map;
uniform sampler2D metallic_map;

uniform samplerCube reflection_map;
uniform vec2 normal_map_size;
uniform vec2 texture_displacement;

uniform bool material_dtex;	// diffuse map
uniform bool material_stex; // specular map
uniform bool material_mtex; // metallic map
uniform bool material_rtex; // roughness map
uniform bool material_htex; // normal map
uniform bool material_ltex; // lightmap
uniform bool material_shadeless;
//uniform float material_wet;

float diffuse,cone;
vec3 reflection;

vec3 normal,lighting;
vec3 specular_factor,specular_mask;
vec4 world_position, color;
uniform mat4 camera_inverted,transform, projection,camera_transform,projection_inverted;
float roughness;

#include "data/shaders/functions/projections.glsl"
#include "data/shaders/lighting/lights.glsl"


void main(void)
{
    world_position = position*projection_inverted*camera_transform;
    
    normal = normals;
    color.rgb = material_dtex ? texture(diffuse_map, tex_map).rgb : material_diffuse;
    
    specular_mask = material_stex ? texture(specular_map, tex_map).rgb*material_specular : material_specular;
    roughness = material_rtex ? length(texture(roughness_map, tex_map).rgb)*material_roughness : material_roughness;
        
    // Фоновое освещение
    lighting = vec3(1.0);
    float lc = 0.3;

    /*lighting+= textureLod(reflection_map, vec3( 0.0, 1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0, 1.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(reflection_map, vec3( 0.0,-1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0,-1.0))+lc)/(1.0+lc), 0.0);
    
    lighting+= textureLod(reflection_map, vec3(-1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3(-1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(reflection_map, vec3( 1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    
    lighting+= textureLod(reflection_map, vec3( 0.0, 0.0,-1.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 1.0, 0.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(reflection_map, vec3( 0.0, 0.0, 1.0), 8.0).rgb * max((dot(normal, vec3( 0.0,-1.0, 0.0))+lc)/(1.0+lc), 0.0);
*/
    if (material_ltex)
    {
        lighting *= texture(light_map, tex_map2).rgb;
    }
    lighting *= 0.5;
    // Сложение всех источников света
    specular_factor = vec3(0.0);
    lighting += sunDiffuse();
    lighting += spotDiffuse();
    lighting += pointDiffuse();
    
    
    fragColor.rgb = max(lighting*color.rgb, 0.0);
    fragColor.a = clamp((texture(diffuse_map,tex_map).a-0.5)*5.0+0.5, 0.0, 1.0);
    if (fragColor.a<0.9)
    {
        discard;
    }
}
