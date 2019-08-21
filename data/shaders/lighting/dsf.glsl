#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

/* Шейдер отложенного затенения */
/* Отложенное затенение позволяет повысить производительность в сценах с большим кол-вом 
 * источников света. Освещение выполняется только для видимых областей.
 */
 
uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gNormals;// Отображённые векторы нормали
uniform sampler2D gNormalsGlass;// Отображённые векторы нормали
uniform sampler2D gAmbient;    // Текстура фонового освещения
uniform sampler2D gMasks;   // Тесктура бликов
uniform sampler3D gVoxelMap;

uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
//uniform sampler2DArray gNoise;
uniform sampler2D text;       // Изображение с текстом, выводимое на экран
uniform samplerCube cubemap;

input  vec2 tex_map;
input  mat3 TBN;
input  vec2 pixel_coord,resolution;

#if __VERSION__ != 100 && __VERSION__ != 110 && __VERSION__ != 120
output vec4 fragColor[2];
#else
  #define fragColor gl_FragData
#endif

uniform float FPS;
uniform int lights_count;
uniform float width,height,zFar,zNear,Angle;
uniform mat4 camera_transform, camera_inverted,projection;

vec4 world_position;
vec3 normal;
vec3 lighting;
vec3 specular_factor;
float specular_mask;
float roughness;
uniform mat4 projection_inv;

#include "data/shaders/functions/projections.glsl"
#include "data/shaders/lighting/lights.glsl"

uniform mat4 vx_camera_transform;
vec4 globalSpace(vec2 screenPosition, float dist)
{
    vec4 pos = vec4((screenPosition*2.0-1.0)*dist, 0.0, 1.0);
    pos*= projection_inv;
    pos.zw = vec2(-dist,1.0);
    pos*= camera_transform;
    pos.w = 1.0;
    return pos;
}

vec3 saturation(vec3 color,float sat)
{
  return mix(vec3(length(color))/sqrt(3.0),color,sat);
}


vec3 overlay(vec3 previousmix, vec3 amount)
{
        const vec3 lumcoeff = vec3(0.2125,0.7154,0.0721);
        
        vec3 luma = vec3(dot(previousmix,lumcoeff));
        
        const vec3 one = vec3(1.0);	
        const vec3 two = vec3(2.0);
	float luminance = dot(previousmix,lumcoeff);
	float mixamount = clamp((luminance - 0.45) * 10., 0., 1.);

	vec3 branch1 = two * previousmix * luma;
	vec3 branch2 = one - (two * (one - min(previousmix,1.0)) * (one - min(luma,1.0)));
	
	vec3 result = mix(branch1, branch2, vec3(mixamount) );

	return mix(previousmix, result, amount);
}
/*
vec3 VXGI()
{
    vec4 val = vec4(0.0);
    int steps = 10;

    float d_start = VOXEL_SIZE*1.42;
    float d_end = VOXEL_MAP_SIZE;
    float d_step = pow(d_end/d_start,1.0/steps);
    vec4 acc = vec4(0.0);
    vec4 pos;
    float Lod = 0.0;
    float dist_inc = 0.0;
    vec3 direction = reflect(normal, normalize(transpose(camera_transform)[3].xyz-world_position.xyz));
    direction = normal;
    for (float dist=d_start;dist<d_end;dist*=d_step)
    {
        Lod = log2(1.0 + dist/VOXEL_SIZE*0.1);
        pos.xyz = world_position.xyz + dist*direction;
        vec3 vs = voxelSpace(pos, VOXEL_MAP_SIZE);
        
        vec4 voxelMapLayer = textureLod(gVoxelMap, vs, clamp(Lod,0.0, 5.0));
        if (voxelMapLayer.a>0)
        {
            val = voxelMapLayer/voxelMapLayer.a;
            val.a = 1.0*pow(2.0,-Lod);
            acc += vec4(val.xyz*val.a,val.a);
        }
        if (acc.a>=1.0)
        {
            return acc.rgb/acc.a / pow(dist+1,2.0);
        }
    }
    return textureLod(cubemap, normal.xzy, 8.0).rgb;
}*/

void main()
{
//      fragColor[1] = fragColor[0] = vec4(tex_map.x>0.5 ? texture(gNormalsGlass,tex_map).rgb : texture(gNormals,tex_map).rgb,1.0);
//      return;

    world_position =  gPosition(gNormals, tex_map, projection_inv, camera_transform);//  globalSpace(tex_map, dep);
    
    vec4 skybox = vec4((tex_map*2.0-1.0)*5.0,0.0,0.0) * projection_inv;
    skybox.zw = vec2(-5.0, 1.0);
    skybox *= mat4(vec4(camera_inverted[0].x,camera_inverted[1].x, camera_inverted[2].x, 0.0),
                   vec4(camera_inverted[0].y,camera_inverted[1].y, camera_inverted[2].y, 0.0),
                   vec4(camera_inverted[0].z,camera_inverted[1].z, camera_inverted[2].z, 0.0),
                   vec4(0.0,0.0,0.0,1.0));
                   
    skybox.xyz = normalize(skybox.xzy * vec3(1.0,1.0,-1.0));
    skybox.rgb = texture(cubemap, skybox.xyz).rgb;
    
    // Извлечение нормалей из текстуры
    normal = texture(gNormals,tex_map).rgb;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    
    vec4 color = texture(gTextures,tex_map);
    if (color.a<0.01)
    {
        fragColor[1].rgb = fragColor[0].rgb = skybox.rgb * pow(length(skybox.rgb), 0.5);
        fragColor[1].a = fragColor[0].a = 1.0;
        return;
    }
    //color *= pow(length(color), 0.9);
    
    specular_mask = texture(gMasks,tex_map).r;
    roughness = mix(10.0, 500.0, texture(gMasks,tex_map).r);
    
    vec4 camera_vector = vec4(normalize(vec3(tex_map-0.5,-1.0)), 0.0);
    camera_vector *= camera_transform;
    
    // Фоновое освещение
    vec3 lighting = vec3(0.0);
    vec3 ambient = texture(gAmbient,tex_map).rgb;
    float lc = 0.3;
    lighting = textureLod(cubemap, vec3( 0.0, 1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0, 1.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(cubemap, vec3( 0.0,-1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0,-1.0))+lc)/(1.0+lc), 0.0);
    
    lighting+= textureLod(cubemap, vec3(-1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3(-1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(cubemap, vec3( 1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    
    lighting+= textureLod(cubemap, vec3( 0.0, 0.0,-1.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 1.0, 0.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(cubemap, vec3( 0.0, 0.0, 1.0), 8.0).rgb * max((dot(normal, vec3( 0.0,-1.0, 0.0))+lc)/(1.0+lc), 0.0);

    
    if (ambient!=vec3(1.0,0.0,1.0))
    {
        lighting *= ambient;// * vec3(.25,0.27,0.35)/0.35;// * 0.0;
    }
    #ifdef _SSGI
        lighting = max(lighting-1.0, 0.0);
    #else
        lighting *= 0.5;
    #endif
    // Сложение всех источников света
    specular_factor = vec3(0.0);
    lighting += sunDiffuse();
    lighting += spotDiffuse();
    lighting += pointDiffuse();
    //lighting = pow(lighting, vec3(1.0/2.1));
    
    fragColor[0].rgb = mix(skybox.rgb, max(lighting*color.rgb + specular_factor, 0.0), color.a);// + god_rays(0.1, 20.0);
    fragColor[0].a = 1.0;
    
    fragColor[1] = fragColor[0];// = god_voxels(0.01,10.0);// = vec4(VXGI(),1.0);
} 
