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
uniform sampler3D gVoxelMap;
uniform samplerCube cubemap;

uniform sampler2D text;
uniform vec3 lSunDirection,lSunPosition;
uniform mat4 camera_transform,camera_inverted,projection,projection_inv;
uniform float width,height,zFar,zNear,Angle;
input  vec2 tex_map;
uniform float time;

// Отражения

#if __VERSION__ == 100 || __VERSION__ == 120
  #define fragColor gl_FragColor
#else
output vec4 fragColor;
#endif

vec4 world_position;
vec3 normal;

#include "data/shaders/raytracing/ssr.glsl"

float gaussian(float x,float sig)
{
    return 1.0/(sig*2.5066282745951782*exp(x*x/(2.0*sig*sig)));
}

vec3 saturation(vec3 color,float coeff)
{
  return mix(color,vec3(length(color)),coeff);
}

float getDepth(vec2 disp)
{
  return texture(gNormals,tex_map+disp).a;
}

void main()
{
    normal = texture(gNormals,tex_map).rgb;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    world_position = gPosition(gNormals, tex_map, projection_inv, camera_transform);
    
    float count = 8;
    float factCount = 0;
    float d = getDepth(vec2(0.0));
    //fragColor.rgb = texture(filtered,tex_map).rgb;return;
    fragColor.rgb = vec3(0.0);
    for (float i=-1.0;i<1.0;i+=2.0/count)
    for (float j=-1.0;j<1.0;j+=2.0/count)
    {
      vec2 disp = vec2(0.5*count*j/width,0.5*count*i/height);
      float dd = getDepth(disp);
      if (abs(dd-d)>0.003) continue;
      factCount += 1;
      vec3 sample1 = texture(filtered,tex_map+disp).rgb;
      fragColor.rgb += sample1;
    }
    fragColor.rgb /= factCount;
    
    //fragColor.rgb = texture(filtered,tex_map).rgb;
    fragColor.rgb *= texture(gTextures,tex_map).rgb;
    fragColor.rgb += texture(original,tex_map).rgb;
}
