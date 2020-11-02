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

uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
uniform sampler2D text;       // Изображение с текстом, выводимое на экран
uniform sampler2D spheremap;
uniform samplerCube cubemap;

input  vec2 tex_map;
input  mat3 TBN;
input  vec2 pixel_coord,resolution;

uniform float FPS;
uniform int lights_count;
uniform float width,height,zFar,zNear;
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

void main()
{
    fragData[1].a = fragData[0].a = 1.0;
    fragData[1].rgb = fragData[0].rgb = texture(gTextures,tex_map).rgb;
    //return;
    world_position =  gPosition(gNormals, tex_map, projection_inv, camera_transform);//  globalSpace(tex_map, dep);
    
    vec4 skybox = vec4((tex_map*2.0-1.0)*5.0,0.0,0.0) * projection_inv;
    skybox.zw = vec2(-5.0, 1.0);
    skybox *= mat4(vec4(camera_inverted[0].x,camera_inverted[1].x, camera_inverted[2].x, 0.0),
                   vec4(camera_inverted[0].y,camera_inverted[1].y, camera_inverted[2].y, 0.0),
                   vec4(camera_inverted[0].z,camera_inverted[1].z, camera_inverted[2].z, 0.0),
                   vec4(0.0,0.0,0.0,1.0));
                   
    skybox.rgb = textureCubemap(cubemap, skybox.xyz).rgb;
    
    // Извлечение нормалей из текстуры
    normal = texture(gNormals,tex_map).rgb;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    
    vec4 color = texture(gTextures,tex_map);
    if (color.a<0.01)
    {
        fragData[1].rgb = fragData[0].rgb = skybox.rgb * pow(length(skybox.rgb), 0.5);
        fragData[1].a = fragData[0].a = 1.0;
        return;
    }
    //color *= pow(length(color), 0.9);
    
    specular_mask = texture(gMasks,tex_map).r;
    roughness = texture(gMasks,tex_map).g;
	float fresnel = texture(gMasks,tex_map).a;
    
    vec4 camera_vector = vec4(normalize(vec3(tex_map-0.5,-1.0)), 0.0);
    camera_vector *= camera_transform;
    

    // Фоновое освещение
    vec3 lighting = vec3(0.0);
    vec3 ambient = texture(gAmbient,tex_map).rgb;
    float lc = 0.3;

    #ifndef _SSGI
    lighting+= textureLod(cubemap, vec3( 0.0, 1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0, 1.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(cubemap, vec3( 0.0,-1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0,-1.0))+lc)/(1.0+lc), 0.0);
    
    lighting+= textureLod(cubemap, vec3(-1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3(-1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(cubemap, vec3( 1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    
    lighting+= textureLod(cubemap, vec3( 0.0, 0.0,-1.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 1.0, 0.0))+lc)/(1.0+lc), 0.0);
    lighting+= textureLod(cubemap, vec3( 0.0, 0.0, 1.0), 8.0).rgb * max((dot(normal, vec3( 0.0,-1.0, 0.0))+lc)/(1.0+lc), 0.0);
    #endif
    
    if (ambient!=vec3(1.0,0.0,1.0))
    {
        lighting *= ambient;
    }
    //lighting *= 0.5;
    
    // Сложение всех источников света
    specular_factor = vec3(0.0);
    lighting += sunDiffuse();
    lighting += spotDiffuse();
    lighting += pointDiffuse();
    
    fragData[0].rgb = mix(skybox.rgb, max(lighting*color.rgb + specular_factor, 0.0), color.a);
    fragData[0].a = 1.0;
    
    fragData[1] = fragData[0];
} 
