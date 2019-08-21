#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/functions/extensions.glsl"

/* Шейдер отложенного затенения */
/* Отложенное затенение позволяет повысить производительность в сценах с большим кол-вом 
 * источников света. Освещение выполняется только для видимых областей.
 */

uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gNormals;// Отображённые векторы нормали
uniform sampler2D gAmbient;    // Текстура фонового освещения
uniform sampler2D gMasks;   // Тесктура бликов

uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
uniform sampler2D text;       // Изображение с текстом, выводимое на экран
uniform samplerCube cubemap;

input  vec2 tex_map;
input  mat3 TBN;
input  vec2 pixel_coord,resolution;

#if __VERSION__ != 100 && __VERSION__ != 110 && __VERSION__ != 120
output vec4 fragColor[5];
#else
  #define fragColor gl_FragData
#endif

uniform float FPS;
uniform int lights_count;
uniform float width,height,zFar,zNear,Angle;
uniform mat4 camera_transform, camera_inverted,projection,projection_inv;

vec3 specular_mask;
mat4 perspective_inv;

#include "data/shaders/functions/projections.glsl"

void main()
{
    //perspective_inv = inverse(perspective(width,height,zNear,zFar,Angle));
    vec4 skybox = vec4((tex_map*2.0-1.0)*5.0,0.0,0.0) * projection_inv;
    skybox.zw = vec2(-5.0, 1.0);
    skybox *= mat4(vec4(camera_inverted[0].x,camera_inverted[1].x, camera_inverted[2].x, 0.0),
                   vec4(camera_inverted[0].y,camera_inverted[1].y, camera_inverted[2].y, 0.0),
                   vec4(camera_inverted[0].z,camera_inverted[1].z, camera_inverted[2].z, 0.0),
                   vec4(0.0,0.0,0.0,1.0));
                   
    skybox.xyz = normalize(skybox.xzy * vec3(1.0,1.0,-1.0));
    
    fragColor[0] = vec4(texture(cubemap, skybox.xyz).rgb, 0.0);
    
    fragColor[1] = fragColor[2] = fragColor[3] = fragColor[4] = vec4(0.0);
} 
 
