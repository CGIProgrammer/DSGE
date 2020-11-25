
struct sPointLight
{
    vec3 position;
    vec4 color;
};

struct sSpotLight
{
    vec3 position;
    vec4 color;
    vec3 direction;
    mat4 itransform;
    float blending;
    float angle_tan;
    bool shadow;
};

struct sSunLight
{
    vec3 direction;
    vec4 color;
    mat4 itransform;
    float depth_range;
    bool shadow;
};

uniform sSunLight lSun;
uniform sSpotLight lSpots[4];
uniform sPointLight lPoints[4];

uniform sampler2D lSpotShadowMaps[4];
uniform samplerCube lPointShadowMaps[4];
uniform sampler2D lSunShadowMap, lSunShadowMapNear, lSunShadowMapNearDyn;
uniform int lSpotCount;
uniform int lPointCount;
uniform int gDitherIteration;

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return num / denom;
}
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}

void lightModel(vec3 color, vec3 N, vec3 L, vec3 V, out vec3 diffuse, out vec3 specular)
{
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
    vec3 numerator    = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
    vec3 spec     = numerator / max(denominator, 0.001);
    float NdotL = max(dot(N, L), 0.0);
    diffuse = kD / PI * radiance * NdotL;
    specular = spec * radiance * NdotL;
}

float lShadowCubeSample(samplerCube smplr, vec3 vector, float fz)
{
    vector = vec3(-vector.x, vector.z, -vector.y);
    vec3 moments = abs(texture(smplr, vector).xyz);
    float E_x2 = moments.y;
    float Ex_2 = moments.x * moments.x;
    float mD = moments.x - fz;
    float variance = (E_x2 - Ex_2);
    float mD_2 = mD*mD;
    float p = variance / (variance + mD_2);
    float shd_s = max(p, float(fz <= moments.x + 0.01));
    float shd_h = float(fz < moments.x+0.003*fz);

    return shd_h;
}

float lShadowSample(sampler2D smplr, vec4 global_coordinates, mat4 itransform, float fz)
{
    global_coordinates *= itransform;
    vec2 crd = global_coordinates.xy/global_coordinates.w * 0.5 + 0.5;
    int cnt = 4;
    float moment = float(fz <= abs(texture(smplr,   crd + blueRand2(tex_map, gDitherIteration*cnt)/1024.0).r)+0.003*fz);
    for (int i=1; i<cnt; i++) 
    {
        moment += float(fz <= abs(texture(smplr, crd + blueRand2(tex_map, i + gDitherIteration*cnt)/1024.0).r)+0.003*fz);
    }
    if (abs(crd.x)<1.0 && abs(crd.y)<1.0)
        return moment / float(cnt);
    else
        return 1.0;
}

mat4 remove_projection(mat4 mat)
{
    mat4 tmat = transpose(mat);
    float m = sqrt(1.0 - tmat[0].w * tmat[0].w) / length(tmat[0].xy);
    
    return transpose(mat4(
        vec4(tmat[0].xy * sqrt(1.0 - tmat[0].w * tmat[0].w) / length(tmat[0].xy), -tmat[0].w, 0.0),
        vec4(tmat[1].xy * sqrt(1.0 - tmat[1].w * tmat[0].w) / length(tmat[1].xy), -tmat[1].w, 0.0),
        vec4(tmat[2].xy * sqrt(1.0 - tmat[2].w * tmat[0].w) / length(tmat[2].xy), -tmat[2].w, 0.0),
        vec4(tmat[3].xy * m, -tmat[3].w, 1.0)
    ));
}

void lSunDiffuse(inout vec3 diffuse, inout vec3 specular)
{
    vec3 spec, diff;
    mat4 itransform = remove_projection(lSun.itransform);
    vec4 kuk = worldPosition*lSun.itransform;
    float lightDistance = (1.0-abs(kuk.z))*lSun.depth_range*0.5/kuk.w;
    float shadow = lShadowSample(lSunShadowMap, worldPosition, lSun.itransform, abs(lightDistance));
    vec3 V = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
    //diffuse += max(dot(mNormal, -normalize(lSun.itransform[2].xyz)), 0.0) * shadow;
    
    lightModel(lSun.color.rgb, mNormal, -normalize(lSun.itransform[2].xyz) * 0.1, V, diff, spec);
    diffuse += diff*shadow;
    specular += spec*shadow;
}

/*void lSunDiffuse(inout vec3 diffuse, inout vec3 specular)
{
    mat4 itransform = remove_projection(lSun.itransform);
    vec4 kuk = worldPosition*lSun.itransform;
    float lightDistance = (1.0-abs(kuk.z))*lSun.depth_range*0.5/kuk.w;
    float shadow = lShadowSample(lSunShadowMap, worldPosition, lSun.itransform, abs(lightDistance));
    diffuse += 5.0 * max(dot(mNormal, -normalize(lSun.itransform[2].xyz)), 0.0) * shadow;
}*/

void lSpotDiffuse(inout vec3 diffuse, inout vec3 specular)
{
    vec3 spec, diff;
    
    if (0<lSpotCount)
    {
        mat4 itransform = remove_projection(lSpots[0].itransform);
        vec4 lightSpacePosition = worldPosition * itransform;
        vec3 dir = lSpots[0].position - worldPosition.xyz;
        vec3 V = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
        float lightDistance = length(dir);
        vec2 coords = lightSpacePosition.xy/lightSpacePosition.z * lSpots[0].angle_tan;
        float spotFactor = smoothstep(1.0, lSpots[0].blending, length(coords));
        float shadow_sample = lShadowSample(lSpotShadowMaps[0], worldPosition, lSpots[0].itransform, lightDistance);
        //shadow_sample = 1.0;
        lightModel(lSpots[0].color.rgb, mNormal, dir, V, diff, spec);
        diffuse += diff*shadow_sample*spotFactor;
        specular += spec*shadow_sample*spotFactor;
    }
}
void lPointDiffuse(inout vec3 diffuse, inout vec3 specular)
{
    vec3 spec, diff;
    
    if (0<lPointCount)
    {
        vec3 spec, diff;
        vec3 V = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
        vec3 dir = lPoints[0].position - worldPosition.xyz;
        float l = length(dir);
        float shadow = lShadowCubeSample(lPointShadowMaps[0], dir+blueRand3(tex_map, 0 + gDitherIteration*4)*0.05, l);
        shadow +=      lShadowCubeSample(lPointShadowMaps[0], dir+blueRand3(tex_map, 1 + gDitherIteration*4)*0.05, l);
        shadow +=      lShadowCubeSample(lPointShadowMaps[0], dir+blueRand3(tex_map, 2 + gDitherIteration*4)*0.05, l);
        shadow +=      lShadowCubeSample(lPointShadowMaps[0], dir+blueRand3(tex_map, 3 + gDitherIteration*4)*0.05, l);
        shadow *= 0.25;
        lightModel(lPoints[0].color.rgb, mNormal, dir, V, diff, spec);
        diffuse  += diff * shadow;
        specular += spec * shadow;
    }
}

