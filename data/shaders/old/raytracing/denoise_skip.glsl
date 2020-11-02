#include "data/shaders/functions/extensions.glsl"
#include "data/shaders/functions/projections.glsl"

uniform sampler2D filtered,original;

uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
uniform sampler2D gNormals;// Отображённые векторы нормали
uniform sampler2D gAmbient;    // Текстура фонового освещения
uniform sampler2D gMasks;   // Тесктура бликов
uniform samplerCube cubemap;

uniform sampler2D text;
uniform vec3 lSunDirection,lSunPosition;
uniform mat4 camera_transform,camera_inverted,projection,projection_inv;
uniform float width,height,zFar,zNear,Angle;
input  vec2 tex_map;
uniform float time;

// Отражения
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
    vec3 original_sample = texture(original,tex_map).rgb;
    if (length(original_sample)>1.5) {
      fragColor.rgb = original_sample;
      return;
    }
    normal = texture(gNormals,tex_map).rgb;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    world_position = gPosition(gNormals, tex_map, projection_inv, camera_transform);
    
    float d = getDepth(vec2(0.0));
    float count = 8.0;
	  float scale = 2.0; //clamp(0.8/abs(d),1.0,3.0);
    float factCount = 0.0;
    vec3 average = vec3(0.0);
    int sampleCount = 0;
    fragColor.rgb = vec3(0.0);
	  vec3 nor = gRenderNormal(gNormals, tex_map);
    for (float i=-scale;i<scale;i+=scale*2.0/count)
    for (float j=-scale;j<scale;j+=scale*2.0/count)
    {
      vec2 disp = vec2(0.5*count*j/width,0.5*count*i/height);
      float dd = getDepth(disp);
      vec3 nor_s = gRenderNormal(gNormals, tex_map + disp);
      float dist = abs(dd-d);
      float coeff = abs(dot(nor, nor_s));
      coeff = pow(coeff, 8.0);
      coeff*= float(dist<0.025);
      factCount += coeff;
      vec3 sample1 = texture(filtered,tex_map+disp).rgb;
      sampleCount++;
      average += sample1;
      sample1 *= coeff;
      fragColor.rgb += sample1;
    }
    if (factCount>0.0) fragColor.rgb /= factCount;
    else fragColor.rgb = average / sampleCount;
    
    //fragColor.rgb = texture(filtered,tex_map).rgb;
    //return;

    fragColor.rgb *= texture(gTextures,tex_map).rgb;
    fragColor.rgb += original_sample;
}
