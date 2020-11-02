#include "data/shaders/functions/extensions.glsl"
#include "data/shaders/functions/projections.glsl"

uniform sampler2D filtered,original;

uniform sampler2D gTextures;   // Отображённые текстуры
uniform sampler2D gVectors;    // Векторы скорости для размытия в движении
uniform sampler2D gNormals,gNormalsGlass;// Отображённые векторы нормали
uniform sampler2D gAmbient;    // Текстура фонового освещения
uniform sampler2D gMasks;   // Тесктура бликов
uniform sampler2D spheremap;
uniform samplerCube cubemap;

uniform vec3 lSunColor;
uniform mat4 lSunTransform;
uniform vec3 lSunDirection;
uniform float lSunFar,lSunNear;
uniform mat4 camera_transform,camera_inverted,projection,projection_inv;
uniform float width,height,zFar,zNear;
input  vec2 tex_map;
uniform float time;

vec4 world_position;
vec3 normal, specular_factor;
float depth,depth_glass,roughness,specular_mask;

// Отражения
#include "data/shaders/raytracing/ssr.glsl"

// Источники света
#include "data/shaders/lighting/lights.glsl"

float gaussian(float x,float sig)
{
    return 1.0/(sig*2.5066282745951782*exp(x*x/(2.0*sig*sig)));
}


float sunShadowSample(vec3 shadowCoords)
{
    float s_res = 2048.0;
    float zNear = lSunNear;
    float zFar = lSunFar;
    
    float z_b =  texture(lSunShadowMap, shadowCoords.xy).r;
    if (shadowCoords.x<=0.0 || shadowCoords.x>=1.0 ||
        shadowCoords.y<=0.0 || shadowCoords.y>=1.0) return 1.0;
    return float(shadowCoords.z<z_b + (zFar-zNear)*0.08e-04);
}

vec3 god_rays(float near,float far)
{
    vec3 camera_vector = normalize(world_position.xyz-transpose(camera_transform)[3].xyz);
    float sunGlareCoeff = max(dot(camera_vector.xyz,normalize(lSunDirection.rgb)),0.0);
    sunGlareCoeff = pow(sunGlareCoeff,5.0);
    if (sunGlareCoeff<0.01)
    {
        return vec3(0.0);
    }
    float val = 0.0;
    int count=1;
    int steps = 50;
    far = min(far, depth);
    float d_step = pow(far/near,1.0/steps);
    vec4 pos = world_position;
    float power = 0.01;
    float dpower= pow(1.0/power,0.1);
    float pdist = near;
    for (float dist=near;dist<far && count<steps;dist*=d_step,count++)
    {
        pos = vec4((tex_map*2.0-1.0)*dist,0.0,1.0);
        pos *= projection_inv;
        pos.z = -dist;
        pos.w = 1.0;
        pos *= camera_transform;
        power = abs(pdist-dist);
        vec4 sh_coords = pos*lSunTransform*bias2;
        val += sunShadowSample(sh_coords.xyz) * power;
        pdist = dist;
    }
    
    val /= steps;
    val *= sunGlareCoeff;
    
    return val*pow(lSunColor, vec3(0.5));
}


vec3 bloom(float threshold)
{
    vec3 bl = vec3(0.0);
    float n=0;
    
    vec2 directions[] = vec2[]
    (
        vec2( 0.5,-1.0),
        vec2(-0.5,-1.0),
        vec2( 0.5, 1.0),
        vec2(-0.5, 1.0),
        vec2( 1.0, 0.5),
        vec2( 1.0,-0.5),
        vec2(-1.0, 0.5),
        vec2(-1.0,-0.5)
    );
    
    for (float lod=1.0;lod<9.0;lod+=1.0)
    {
        vec2 res = 1.0/vec2(width, height)*pow(2.0, lod);
        for (int i=0;i<8;i++)
        {
            vec3 s = textureLod(original,tex_map+res*directions[i]*0.7, lod).rgb * (1.0+(lod-1.0)*0.5);
            float l = length(s);
            if (l==0.0)
            {
                continue;
            }
            s = normalize(s);
            bl += s * max(l-threshold/lod, 0);
        }
        n+=8;
    }
    return bl/n;
}

vec3 motion_blur()
{
  vec3 blur = vec3(0.0);
  float n=0;
  float i;
  vec2 speed = texture(gVectors,tex_map).rg;//*0.25;
  for (i=-0.25;i<0.25;i+=0.02)
  {
      blur+=texture(original,tex_map+speed*i).rgb;
      n+=1.0;
  }
  return blur / n;
}
vec2 pixeled;
float vignette()
{
  return 0.1/(pow(length(tex_map-0.5),5.0)+0.1);
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
    fragColor.rgb = texture(filtered,tex_map).rgb;
    //fragColor.rgb = motion_blur();
    normal = texture(gNormalsGlass,tex_map).rgb;
//     fragColor = vec4(normal, 1.0);
//     return;
    normal = normalize((normal-(0.5+1.0/400.0))/(0.5-1.0/200.0));
    depth = 1.0/texture(gNormals,tex_map).a - 1.0;
    depth_glass = 1.0/texture(gNormalsGlass,tex_map).a - 1.0;
    world_position = gPosition(gNormalsGlass, tex_map, projection_inv, camera_transform);
    fragColor.rgb = mix(texture(original, tex_map).rgb, max(fragColor.rgb, 0.0), texture(gTextures, tex_map).a);
    vec3 camera_vector = normalize(world_position.xyz-transpose(camera_transform)[3].xyz);
    vec4 specularColor = texture(gMasks,tex_map);
    
    bool glass = depth_glass<depth;
	
	float specular = specularColor.x;
	float roughness = 1.0;//specularColor.y;
	float metallic = specularColor.z;
	float fresnel = specularColor.a;
    
    if (metallic>0.0 || glass)
    {
        float HV = dot(-camera_vector, normal);
        float F  = pow(1.0 - HV, fresnel);// * 1.0-clamp(fresnel, 0.0, 1.0);
      
        vec3 reflectedVector = reflect(camera_vector,normal);
        vec4 ssr = SSR(world_position.xyz, reflectedVector, 75.0, roughness);
        
        vec3 diffuseColor = texture(gTextures,tex_map).rgb;
        if (glass)
        {
			vec3 refractedVector = refract(camera_vector,normal, 1.0/1.4);
            vec4 ssr2 = SSR(world_position.xyz, refractedVector, 20.0, roughness);
            ssr2.xyz = mix(vec3(0.1,0.2,0.3), ssr2.xyz, 1.0/(ssr2.a+1.0));
            specular_mask = 1.0;
            roughness = 170.0;
            ssr.rgb += glare(lSunDirection) * 40.0;
            fragColor.rgb = mix(ssr2.rgb, ssr.rgb, clamp(F,0.0,1.0) * 0.5);
        }
        else
        {
            fragColor.rgb = mix(fragColor.rgb, ssr.rgb, clamp(F*metallic*2.0,0.0,1.0));
        }
    }
    //fragColor.rgb = pow(fragColor.rgb, vec3(0.75));
    //fragColor.rgb += god_rays(1.0, 20.0);
}
