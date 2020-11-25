#include "data/shaders/functions/extensions.glsl"
#include "data/shaders/functions/projections.glsl"

uniform sampler2D gOutput, gMasks, gAlbedo;

uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gSpace;// Отображённые векторы нормали

uniform int gDitherIteration;
uniform sampler2D text;
uniform vec3 lSunDirection,lSunPosition;
uniform mat4 camera_transform,camera_inverted,projection,projection_inv;
uniform float zFar,zNear,Angle;
uniform vec2 gResolution;
input  vec2 tex_map;
uniform float time;

// Отражения
vec4 world_position;
vec3 normal;

void main()
{
    normal = texture(gSpace,tex_map).rgb;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    world_position = gPosition(gSpace, tex_map, projection_inv, camera_transform);
    
    float d = unpack_depth(texture(gSpace, tex_map).a);
    const float scale = 2.0;// * (1.0+fract(gDitherIteration)); //clamp(0.8/abs(d),1.0,3.0);
    const float count = 5.0;
    float factCount = 0.0;
    fragColor.rgb = vec3(0.0);
    vec3 nor = gRenderNormal(gSpace, tex_map);
    float r0 = texture(gMasks, tex_map).g;
    for (float i=-scale + mod(float(gDitherIteration), 2.0); i<=scale; i+=scale*2.0/count)
    for (float j=-scale + mod(float(gDitherIteration), 2.0); j<=scale; j+=scale*2.0/count)
    {
      vec2 disp = 0.5*count * vec2(j, i) / gResolution;
      float dd = unpack_depth(texture(gSpace, tex_map + disp).a);
      vec3 nor_s = gRenderNormal(gSpace, tex_map + disp);
      float roughness = min(abs(texture(gMasks, tex_map + disp).g - r0) * 10.0, 1.0);
      float metallic = min(abs(texture(gMasks, tex_map + disp).b) * 10.0, 1.0);
      float dist = abs(dd-d);
      float coeff = abs(dot(nor, nor_s));
      coeff = pow(coeff, 8.0) * (1.0-roughness);// * (1.0-metallic);
      coeff*= float(dist<0.05);
      factCount += coeff;
      vec3 sample1 = texture(gOutput,tex_map+disp).rgb;
      fragColor.rgb += sample1 * coeff;
    }
    if (factCount==0.0) {
        fragColor.rgb = texture(gOutput,tex_map).rgb * texture(gAlbedo,tex_map).rgb;
    } else {
        fragColor.rgb = fragColor.rgb / factCount * texture(gAlbedo,tex_map).rgb;
    }
}
