#include "data/shaders/version.glsl"
#include "data/shaders/head.h"
#include "data/shaders/config.h"
#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
input vec2 tex_map2;
input vec3 normals,viewDir,viewDirTBN;
input vec4 position;

const int TEXTURE = 0;
const int NORMAL = 1;
const int AMBIENT = 2;
const int SPECULAR = 3;
const int NORMAL_GLASS = 4;

input mat3 TBN;
#if __VERSION__ == 120 || __VERSION__ == 100
#define fragColor gl_FragData
#else
output vec4 fragColor[5];
#endif

vec2 velocity_vector;

uniform int lights_count;
uniform vec3 material_diffuse;
uniform vec3 material_specular;
uniform float transparency;
uniform float height_scale;
uniform float reflection_coeff;

uniform sampler2D diffuse_map;
uniform sampler2D light_map;
uniform sampler2D specular_map;
uniform sampler2D normal_map;
uniform samplerCube reflection_map;
uniform vec2 normal_map_size;
uniform vec2 texture_displacement;

uniform float material_glow;
uniform bool material_dtex;
uniform bool material_ntex;
uniform bool material_stex;
uniform bool material_htex;
uniform bool material_ltex;
uniform float material_wet;
uniform bool material_shadeless;

float diffuse,cone,specular;
vec3 reflection;

vec2 spheremap(vec3 vector)
{
  vector = normalize(vector);
  vec2 result;

  result.y = 1.0-(vector.z*0.5+0.5);
  vector.xy/=sqrt(1.0-vector.z*vector.z);
  result.x = acos(vector.y)/3.1415926535/2.0;
  result.x *= vector.x<0.0 ? 1.0 : -1.0;
  result.x = (result.x + 0.5);
  return result;
}

vec3 normal_from_height(sampler2D tex,vec2 coords)
{
    vec2 delta = 0.5/(normal_map_size);
    float x = (texture(tex,coords-vec2(delta.x,0.0)).r-texture(tex,coords+vec2(delta.x,0.0)).r);
    float y = (texture(tex,coords+vec2(0.0,delta.y)).r-texture(tex,coords-vec2(0.0,delta.y)).r);
    //float z = sqrt(1.0-x*x-y*y);
    return normalize(vec3(x,y,0.4));
}

vec3 normal_map_from_rg(sampler2D tex,vec2 coords)
{
    vec3 result = texture(tex, coords).rgb*2.0-1.0;
    result.b = sqrt(1.0 - result.r*result.r - result.g*result.g);
    return normalize(result);
}

vec2 parallax(vec2 texCoords,float scale){
  float numSteps  = 64.0;
  float   step   = 1.0 / numSteps;
  vec2    dtex   = viewDirTBN.xy * scale / ( numSteps * viewDirTBN.z );
  dtex = vec2(dtex.x,-dtex.y);
  float   height = 1.0;
  vec2    tex    = texCoords;
  float h       = texture(normal_map,tex).b;
  while ( height > h )
  {
    height -= step;
    tex    -= dtex;
    h       = texture(normal_map,tex).b;
  }
  return tex;
}

vec3 texture_cube(samplerCube sampler,vec3 coords)
{
  return texture(sampler,vec3(coords.x, coords.z, -coords.y)).rgb;
}

vec3 normal;
vec2 texture_UV;

float calcDepth(float add_coeff)
{
    float far = gl_DepthRange.far;
    float near = gl_DepthRange.near;
    return (((far-near) * position.z/(position.w+add_coeff)) + near + far) / 2.0;
}

void main(void)
{
    vec3 color,specular;
    
    texture_UV = vec2(tex_map+texture_displacement);
    vec2 tex_map_orig = texture_UV;
    fragColor[TEXTURE] = vec4(0.0,0.0,0.0,1.0);
    fragColor[SPECULAR] = vec4(0.0,0.0,0.0,1.0);
    //gl_FragDepth = calcDepth(0.0);
    if (material_htex)
    {
        //texture_UV = parallax(texture_UV,height_scale*0.05);
        color = texture(diffuse_map,texture_UV).rgb;
        specular = texture(specular_map,texture_UV).rgb;
        normal = normal_map_from_rg(normal_map,texture_UV)*TBN;
    }
    else
    {
        color = texture(diffuse_map,texture_UV).rgb;
        specular = texture(specular_map,texture_UV).rgb;
        normal = normals;
    }
        
    vec3 ambient = vec3(1.0,0.0,1.0);//clamp(vec3(dot(vec3(0.0,0.0,1.0),normal)*0.05+0.15)*6.0,0.0,1.0);
    if (material_ltex)
    {
        ambient = texture(light_map,tex_map2).rgb;//pow(texture(light_map,tex_map2).rgb,vec3(1.2));
    }
    fragColor[SPECULAR].rgb  = material_stex ? specular*material_specular : material_specular;
    fragColor[NORMAL  ].rgb  = normalize(normal.rgb)*(0.5-1.0/200.0)+0.5+1.0/400.0;
    fragColor[NORMAL  ].a    = 1.0/(position.w+1.0);
    #ifdef _SSR
    fragColor[NORMAL_GLASS] = fragColor[NORMAL];
    #endif
    fragColor[AMBIENT ].rgb  = ambient.rgb;//pow(ambient.rgb, vec3(1.0));
    fragColor[AMBIENT ].rgb += material_glow;
    fragColor[TEXTURE ].rgb  = material_dtex ? color : material_diffuse;
    
    float wet_coeff = material_wet * (1.0 - texture(normal_map, tex_map).z);
    if (material_wet>0.0)
    {
        fragColor[TEXTURE].rgb *= mix(1.0, 0.3, min(wet_coeff, 1.0));
        float lum = length(fragColor[TEXTURE].rgb);
        fragColor[TEXTURE ].rgb *= pow(lum, 0.3*material_wet);
        fragColor[SPECULAR].r    = max(wet_coeff*2.0-1.0, 0.0);
        if (material_wet>0.8)
        {
            normal = normals;
        }
        //fragColor[SPECULAR].g    = mix(fragColor[SPECULAR].g, 300.0, max(wet_coeff-1, 0.0));
    }
    
    fragColor[AMBIENT].a = fragColor[TEXTURE].a = fragColor[SPECULAR].a = 1.0;
    //if (transparency==1.0)
    {
        if (clamp((texture(diffuse_map,texture_UV).a-0.35)*5.0,0.0,1.0)<0.95)
        {
            discard;
        }
    }
        
}
