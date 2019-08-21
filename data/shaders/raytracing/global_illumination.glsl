#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"
#include "data/shaders/functions/projections.glsl"

uniform sampler2D filtered,original;

uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
uniform sampler2D gNormals;// Отображённые векторы нормали
uniform sampler2D gAmbient;    // Текстура фонового освещения
uniform sampler2D gMasks;   // Тесктура бликов
uniform sampler2DArray gNoise;
uniform samplerCube cubemap;
uniform sampler3D gVoxelMap;

uniform sampler2D text;
uniform vec3 lSunDirection,lSunPosition;
uniform mat4 camera_transform,camera_inverted,projection,projection_inv;
uniform float width,height,zFar,zNear,Angle;
input  vec2 tex_map;
uniform float time;

// Отражения
#include "data/shaders/functions/random.glsl"
#include "data/shaders/raytracing/ssr.glsl"

#if __VERSION__ == 100 || __VERSION__ == 120
  #define fragColor gl_FragColor
#else
output vec4 fragColor;
#endif

vec2 pixeled;
vec3 normal;
float GI_coeff;
vec4 world_position;

vec3 SSGI(vec3 rayHit, vec3 norm)
{
    vec3 result;
    float samplesCount = 10.0;
    float samples = 0.0;
    vec3 static_ambient = vec3(0.0);
    float lc = 0.3;
  
    static_ambient+= textureLod(cubemap, vec3( 0.0, 1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0, 1.0))+lc)/(1.0+lc), 0.0);
    static_ambient+= textureLod(cubemap, vec3( 0.0,-1.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 0.0,-1.0))+lc)/(1.0+lc), 0.0);
    
    static_ambient+= textureLod(cubemap, vec3(-1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3(-1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    static_ambient+= textureLod(cubemap, vec3( 1.0, 0.0, 0.0), 8.0).rgb * max((dot(normal, vec3( 1.0, 0.0, 0.0))+lc)/(1.0+lc), 0.0);
    
    static_ambient+= textureLod(cubemap, vec3( 0.0, 0.0,-1.0), 8.0).rgb * max((dot(normal, vec3( 0.0, 1.0, 0.0))+lc)/(1.0+lc), 0.0);
    static_ambient+= textureLod(cubemap, vec3( 0.0, 0.0, 1.0), 8.0).rgb * max((dot(normal, vec3( 0.0,-1.0, 0.0))+lc)/(1.0+lc), 0.0);
    static_ambient *= 0.5;
    vec3 static_ambient_texture = texture(gAmbient,tex_map).rgb;
    if (static_ambient_texture!=vec3(1.0,0.0,1.0))
    {
        static_ambient *= static_ambient_texture;
    }
    
    for (int i=0;i<samplesCount;i++)
    {
        vec3 dir = normalize(normalize(blueRand3(tex_map, i)-0.5) + norm*1.0);
        if (dot(dir, norm) < 0.0)
        {
            dir *= -1.0;
        }
        vec3 cbm = static_ambient;
        vec4 ssrt;
        ssrt = SSRT2(rayHit,dir);
        result += mix(cbm,ssrt.rgb*max(dot(norm,dir),0.0),clamp(ssrt.a, 0.0,1.0));
        samples += ssrt.a;
    }
    return result/samplesCount;
}

void main()
{
    fragColor = texture(original,tex_map);
    world_position = gPosition(gNormals, tex_map, projection_inv, camera_transform);
    
    normal = texture(gNormals,tex_map).rgb;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    vec3 camera_vector = normalize(world_position.xyz-transpose(camera_transform)[3].xyz);
    fragColor.rgb = SSGI(world_position.xyz,normal);
}
