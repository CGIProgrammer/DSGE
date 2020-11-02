#!/usr/bin/python3
from sys import argv

shader_header = '''
struct sPointLight
{{
    vec3 position;
    vec4 color;
}};

struct sSpotLight
{{
    vec3 position;
    vec4 color;
    vec3 direction;
    mat4 itransform;
    float blending;
    float angle_tan;
    bool shadow;
}};

struct sSunLight
{{
    vec3 direction;
    vec4 color;
    mat4 itransform;
    float depth_range;
    bool shadow;
}};

uniform sSunLight lSun;
uniform sSpotLight lSpots[{MAX_LIGHTS}];
uniform sPointLight lPoints[{MAX_LIGHTS}];

uniform sampler2D lSpotShadowMaps[{MAX_LIGHTS}];
uniform samplerCube lPointShadowMaps[{MAX_LIGHTS}];
uniform sampler2D lSunShadowMap, lSunShadowMapNear, lSunShadowMapNearDyn;
uniform int lSpotCount;
uniform int lPointCount;

vec3 specular_factor = vec3(0.0);

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}}
float DistributionGGX(vec3 N, vec3 H, float roughness)
{{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return num / denom;
}}
float GeometrySchlickGGX(float NdotV, float roughness)
{{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}}

vec3 lightModel(vec3 color, vec3 N, vec3 L, vec3 V)
{{
    float distance    = length(L);  
    float attenuation = 1.0 / (1.0 + distance*distance*0.1);
    vec3 radiance     = color * attenuation;      
    L /= distance;
    vec3 H = normalize(V + L);
    float NDF = DistributionGGX(N, H, mRoughness);        
    float G   = GeometrySmith(N, V, L, mRoughness);      
    vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);       
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - mMetallic;	  
    vec3 numerator    = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
    vec3 specular     = numerator / max(denominator, 0.001);
    float NdotL = max(dot(N, L), 0.0);
    return (kD * mDiffuse.xyz / PI + specular) * radiance * NdotL; 
}}

float lShadowCubeSample(samplerCube smplr, vec3 vector, float fz)
{{
    vector = vec3(-vector.x, vector.z, -vector.y);
    vec3 moments = abs(texture(smplr, vector).xyz);
    //return float(moments.x+0.1>fz);
    float E_x2 = moments.y;
    float Ex_2 = moments.x * moments.x;
    float mD = moments.x - fz;
    float variance = (E_x2 - Ex_2); //*clamp(-mD*2.5, 0.1, 1.0);
    float mD_2 = mD*mD;
    float p = variance / (variance + mD_2);
    float shd_s = max(p, float(fz <= moments.x + 0.01));
    float shd_h = float(fz < moments.z + 0.02);

    return min(shd_s, 1.0); //min(shd_s, shd_h);
}}

float lShadowSample(sampler2D smplr, vec4 global_coordinates, mat4 itransform, float fz)
{{
    global_coordinates *= itransform;
    vec2 crd = global_coordinates.xy/global_coordinates.w;
    vec3 moments = abs(texture(smplr, crd*0.5+0.5).xyz);
    float E_x2 = moments.y;
    float Ex_2 = moments.x * moments.x;
    float mD = moments.x - fz;
    float variance = (E_x2 - Ex_2); //*clamp(-mD*2.5, 0.1, 1.0);
    float mD_2 = mD*mD;
    float p = variance / (variance + mD_2);
    float shd_s = max(p, float(fz <= moments.x + 0.05));
    float shd_h = float(fz < moments.z + 0.02);

    if (abs(crd.x)<1.0 && abs(crd.y)<1.0)
        return min(shd_s, shd_h);
    else
        return 1.0;
}}

mat4 remove_projection(mat4 mat)
{{
    mat4 tmat = transpose(mat);
    float m = sqrt(1.0 - tmat[0].w * tmat[0].w) / length(tmat[0].xy);
    
    return transpose(mat4(
        vec4(tmat[0].xy * sqrt(1.0 - tmat[0].w * tmat[0].w) / length(tmat[0].xy), -tmat[0].w, 0.0),
        vec4(tmat[1].xy * sqrt(1.0 - tmat[1].w * tmat[0].w) / length(tmat[1].xy), -tmat[1].w, 0.0),
        vec4(tmat[2].xy * sqrt(1.0 - tmat[2].w * tmat[0].w) / length(tmat[2].xy), -tmat[2].w, 0.0),
        vec4(tmat[3].xy * m, -tmat[3].w, 1.0)
    ));
}}
vec3 lSunDiffuse()
{{
    mat4 itransform = remove_projection(lSun.itransform);
    float phongDiffuse = max(dot(mNormal, -normalize(lSun.itransform[2].xyz)), 0.0);
    vec4 kuk = worldPosition*lSun.itransform;
    float lightDistance = (1.0-abs(kuk.z))*lSun.depth_range*0.5/kuk.w;
    return lSun.color.rgb * phongDiffuse * lShadowSample(lSunShadowMap, worldPosition, lSun.itransform, abs(lightDistance));
}}
vec3 lSpotDiffuse()
{{
    vec3 spot_sample = vec3(0.0);
    {spot_iterations}
    return spot_sample;
}}
vec3 lPointDiffuse()
{{
    vec3 point_sample = vec3(0.0);
    {point_iterations}
    return point_sample;
}}
'''

spot_light_iteration = '''
    if ({index}<lSpotCount)
    {{
        mat4 itransform = remove_projection(lSpots[{index}].itransform);
        vec4 lightSpacePosition = worldPosition * itransform;
        vec3 dir = lSpots[{index}].position - worldPosition.xyz;
        vec3 V = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
        float lightDistance = -lightSpacePosition.z;
        vec2 coords = lightSpacePosition.xy/lightSpacePosition.z * lSpots[{index}].angle_tan;
        float spotFactor = smoothstep(1.0, lSpots[{index}].blending, length(coords));
        float shadow_sample = lShadowSample(lSpotShadowMaps[{index}], worldPosition, lSpots[{index}].itransform, lightDistance);
        spot_sample += lightModel(lSpots[{index}].color.rgb, mNormal, dir, V) * spotFactor * shadow_sample;
    }}
'''

point_light_iteration = '''
    if ({index}<lPointCount)
    {{
        vec3 V = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
        vec3 dir = lPoints[{index}].position - worldPosition.xyz;
        float l = length(dir);
        float shadow = lShadowCubeSample(lPointShadowMaps[{index}], dir+blueRand3(tex_map, 0)*0.05, l-0.05);
        shadow *= 0.25;
        shadow += lShadowCubeSample(lPointShadowMaps[{index}], dir+blueRand3(tex_map, 1)*0.05, l*0.95) * 0.25;
        shadow += lShadowCubeSample(lPointShadowMaps[{index}], dir+blueRand3(tex_map, 2)*0.05, l*0.95) * 0.25;
        shadow += lShadowCubeSample(lPointShadowMaps[{index}], dir+blueRand3(tex_map, 3)*0.05, l*0.95) * 0.25;
        point_sample += lightModel(lPoints[{index}].color.rgb, mNormal, dir, V)*shadow;
    }}
'''
MAX_LIGHTS = 4
if '-ml' in argv:
    MAX_LIGHTS = int(argv[argv.index('-ml')+1])
lSpotDffuse = ''.join([spot_light_iteration.format(index=i) for i in range(MAX_LIGHTS)])
lPointDiffuse = ''.join([point_light_iteration.format(index=i) for i in range(MAX_LIGHTS)])
shader = shader_header.format(spot_iterations=lSpotDffuse, point_iterations=lPointDiffuse, MAX_LIGHTS=MAX_LIGHTS)
print(shader)