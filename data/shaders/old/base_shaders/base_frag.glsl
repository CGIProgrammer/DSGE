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

vec2 velocity_vector;

uniform int lights_count;
uniform vec3 material_diffuse;
uniform vec3 material_specular;
uniform float material_fresnel;
uniform float material_metallic;
uniform float material_roughness;
uniform float material_glow;

uniform float transparency;
uniform float height_scale;
uniform float reflection_coeff;

uniform sampler2D diffuse_map;
uniform sampler2D light_map;
uniform sampler2D specular_map;
uniform sampler2D roughness_map;
uniform sampler2D normal_map;
uniform sampler2D metallic_map;

uniform samplerCube reflection_map;
uniform vec2 normal_map_size;
uniform vec2 texture_displacement;

uniform bool material_dtex;	// diffuse map
uniform bool material_stex; // specular map
uniform bool material_mtex; // metallic map
uniform bool material_rtex; // roughness map
uniform bool material_htex; // normal map
uniform bool material_ltex; // lightmap
uniform bool material_shadeless;
//uniform float material_wet;

float diffuse,cone,specular;
vec3 reflection;

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
    fragData[TEXTURE] = vec4(0.0,0.0,0.0,1.0);
    fragData[SPECULAR] = vec4(0.0,0.0,0.0,1.0);
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
        
    vec3 ambient = clamp(vec3(dot(vec3(0.0,0.0,1.0),normal)*0.05+0.15)*6.0,0.0,1.0);
    if (material_ltex)
    {
        ambient = texture(light_map,tex_map2).rgb;//pow(texture(light_map,tex_map2).rgb,vec3(1.2));
    }
    ambient += material_glow;
    fragData[SPECULAR].r = length(material_stex ? specular*material_specular : material_specular);
	fragData[SPECULAR].g = material_rtex ? length(texture(roughness_map, tex_map).rgb)*material_roughness : material_roughness;
	fragData[SPECULAR].b = material_mtex ? length(texture(metallic_map, tex_map).rgb)*material_metallic : material_metallic;
    fragData[SPECULAR].b = fragData[SPECULAR].b*fragData[SPECULAR].b;
	fragData[SPECULAR].a = material_fresnel;
	
    fragData[NORMAL  ].rgb  = normalize(normal.rgb)*(0.5-1.0/200.0)+0.5+1.0/400.0;
    fragData[NORMAL  ].a    = 1.0/(position.w+1.0);
    #ifdef _SSR
    fragData[NORMAL_GLASS] = fragData[NORMAL];
    #endif
    fragData[AMBIENT ].rgb  = ambient.rgb;
    fragData[TEXTURE ].rgb  = material_dtex ? color : material_diffuse;
    
    /*float wet_coeff = material_wet * (1.0 - texture(normal_map, tex_map).z);
    if (material_wet>0.0)
    {
        fragData[TEXTURE].rgb *= mix(1.0, 0.3, min(wet_coeff, 1.0));
        float lum = length(fragData[TEXTURE].rgb);
        fragData[TEXTURE ].rgb *= pow(lum, 0.3*material_wet);
        fragData[SPECULAR].r    = max(wet_coeff*2.0-1.0, 0.0);
        if (material_wet>0.8)
        {
            normal = normals;
        }
        //fragData[SPECULAR].g  = mix(fragData[SPECULAR].g, 300.0, max(wet_coeff-1, 0.0));
    }*/
    
    fragData[AMBIENT].a = fragData[TEXTURE].a = 1.0;
    //if (transparency==1.0)
    {
        if (clamp((texture(diffuse_map,texture_UV).a-0.35)*5.0,0.0,1.0)<0.95)
        {
            discard;
        }
    }
        
}
