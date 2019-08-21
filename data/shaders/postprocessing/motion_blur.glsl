#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

uniform sampler2D filtered,original;

uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
uniform sampler2D gNormals;// Отображённые векторы нормали
uniform sampler2D gAmbient;    // Текстура фонового освещения
uniform sampler2D gMasks;   // Тесктура

input  vec2 tex_map;

uniform float width,height,zFar,zNear,Angle;

#if __VERSION__ == 100 || __VERSION__ == 120
  #define fragColor gl_FragColor
#else
output vec4 fragColor;
#endif

float gauss(float x)
{
    const float a = sqrt(2.0*3.1415926535);
    return a * exp(-x*x/2.0);
}

vec3 motion_blur()
{
    float d = texture(gNormals,tex_map).a;
    vec3 blur = vec3(0.0);
    float n=0;
    float i;
    vec2 speed = texture(gVectors,tex_map).rg * 0.35;
    for (i=-0.5;i<=0.5;i+=0.05)
    {
        float dd = texture(gNormals,tex_map+speed*i).a;
        vec2 d_speed = texture(gVectors,tex_map+speed*i).rg;
         if (dot(normalize(speed), normalize(d_speed))<0.99 && abs(dd-d)>0.01)
             continue;
        float co = gauss(i*2.0);
        blur+=texture(filtered,tex_map+speed*i).rgb * co;
        n+=co;
    }
    if (n==0.0)
        return texture(filtered, tex_map).rgb;
    return blur / n;
}

void main()
{
    fragColor.rgb = motion_blur();
}
