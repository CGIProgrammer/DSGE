#ifndef PBR_STRUCTURES
#define PBR_STRUCTURES
struct PBRSurface {
    vec3 position;
    vec3 albedo;
    vec3 normal;
    float specular;
    float roughness;
    float metalness;
};

/*
 * Код из статей Learn OpenGL
 */

float DistributionGGX(float NdotH, float roughness)
{
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH2 = NdotH * NdotH;

    float denom = NdotH2 * (a2 - 1.0) + 1.0;
    denom = PI * denom * denom;
    if (NdotH2 > 0.99999 && roughness < 0.01) {
        return 0.0;
    }
    return a2 / denom;
}
/*
def DistributionGGX(N, H, roughness):
    a = roughness * roughness
    a2 = a * a
    NdotH = max(dot(N, H), 0.0)
    NdotH2 = NdotH * NdotH
    denom = NdotH2 * (a2 - 1.0) + 1.0
    denom = PI * denom * denom
    return a2 / denom
    */
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}
float GeometrySmith(float NdotV, float NdotL, float roughness)
{
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    return ggx1 * ggx2;
} 

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void refl(vec3 lightColor, vec3 nV, vec3 L, PBRSurface surface, out vec3 diffuse_reflection, out vec3 specular_reflection)
{
    // расчет энергетической яркости для каждого источника света
    float distance = length(L);
    vec3 nL = L / distance;
    vec3 H = normalize(nV + normalize(L));
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance     = lightColor * attenuation;

    float NdotL = max(dot(surface.normal, L), 0.0);
    float NdotH = max(dot(surface.normal, H), 0.0);
    float NdotV = max(dot(surface.normal, nV), 0.0);
    float VdotH = max(dot(nV, H), 0.0);      
    
    // Cook-Torrance BRDF
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, surface.albedo, surface.metalness);
    vec3 F = fresnelSchlick(VdotH, F0);
    float NDF = DistributionGGX(NdotH, surface.roughness);
    float G = GeometrySmith(NdotV, NdotL, surface.roughness);
    
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - surface.metalness;	  
    
    vec3 numerator    = NDF * G * F;
    float denominator = 4.0 * max(dot(surface.normal, nV), 0.0) * max(dot(surface.normal, L), 0.0);
    vec3 specular     = numerator / (denominator + 0.001);
        
    // прибавляем результат к исходящей энергетической яркости Lo
    vec3 Lo = (kD * surface.albedo / PI + specular) * radiance * NdotL;
    diffuse_reflection = kD * INV_PI * radiance * NdotL;
    specular_reflection = specular * radiance * NdotL;
}

// Конец кода из статей Learn OpenGL
#endif