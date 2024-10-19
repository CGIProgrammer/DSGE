#ifndef PBR_GLSL
#define PBR_GLSL

#ifndef CONSTANTS_GLSL
#define PI 3.1415926535
#define INV_PI 0.3183098861837907
#endif

#include "../random.glsl"
#include "shadowmap.h"
#include "pbr_structures.h"


void spotlight(Spotlight light, PBRSurface surface, vec3 nV, out vec3 diffuse, out vec3 specular)
{
    vec3 l = light.base.location.xyz - surface.position;
    vec3 intensity = light.base.color.rgb * light.base.power;
    float dist = length(l);
    vec3 ln = l / dist;
    refl(light.base.color.rgb*light.base.power, nV, l, surface, diffuse, specular);
    float cos_a = dot(ln, -light.direction.xyz);
    float spot_limit = smoothstep(cos(light.outer_angle), cos(light.inner_angle), cos_a);
    diffuse *= spot_limit;
    specular *= spot_limit;
}

void spotlight(Spotlight light, sampler2DArray shadowmap, PBRSurface surface, vec3 nV, int noise_seed, out vec3 diffuse, out vec3 specular)
{
    int index = light.base.shadowmap_index;
    spotlight(light, surface, nV, diffuse, specular);
    vec3 l = surface.position - light.base.location.xyz;
    float dist = dot(l, light.direction.xyz);
    vec3 ln = normalize(l);
    float shadow = shadow_sample(
        shadowmap,
        index,
        light.base.projection_inv,
        surface.normal,
        surface.position,
        ln,
        dist,
        light.base.distance,
        light.base.z_near,
        noise_seed
    );
    diffuse *= shadow;
    specular *= shadow;
}

void sunlight(SunLight light, PBRSurface surface, vec3 nV, out vec3 diffuse, out vec3 specular)
{
    refl(light.base.color.rgb*light.base.power, nV, -light.direction.xyz, surface, diffuse, specular);
}

void sunlight(SunLight light, sampler2DArray shadowmap, PBRSurface surface, vec3 nV, int noise_seed, out vec3 diffuse, out vec3 specular)
{
    int index = light.base.shadowmap_index;
    sunlight(light, surface, nV, diffuse, specular);
    float dist = dot(light.direction.xyz, surface.position-light.base.location.xyz);
    vec4 shadowmap_homogeneous = light.base.projection_inv * vec4(surface.position, 1.0);
    
    float light_range = light.base.distance - light.base.z_near;
    float min_bias = light_range / 32768.0;
    float max_bias = min_bias * 100.0;
    float cos_angle = sqrt(dot(light.direction.xyz, -surface.normal));
    
    vec2 sm_size = 2.0 / textureSize(shadowmap, 0).xy;
    vec2 sm_coords = shadowmap_homogeneous.xy / shadowmap_homogeneous.w * 0.5 + 0.5;
    int samples = 1;
    
    float shadow = 0.0;
    for (int s=0; s<samples; s++) {
        vec2 bn = bluerand4(blue_noise, s + samples*noise_seed).xy;
        float bias = mix(max_bias+max_bias*length(bn), min_bias, cos_angle);
        float shadow_sample = texture(shadowmap, vec3(sm_coords + bn * sm_size, index)).r;
        shadow_sample = shadow_sample * light_range * 0.5 + light.base.z_near;
        shadow += float(shadow_sample > dist - bias);
    }
    shadow /= samples;
    diffuse *= shadow;
    specular *= shadow;
}

void pointlight(PointLight light, PBRSurface surface, vec3 nV, out vec3 diffuse, out vec3 specular)
{
    vec3 L = light.base.location.xyz - surface.position;
    refl(light.base.color.rgb, nV, L, surface, diffuse, specular);
}

void pointlight(PointLight light, sampler2DArray shadowmaps, PBRSurface surface, vec3 nV, int noise_seed, out vec3 diffuse, out vec3 specular)
{
    int idx = light.base.shadowmap_index;
    vec3 L = light.base.location.xyz - surface.position;
    refl(light.base.color.rgb, nV, L, surface, diffuse, specular);

    const int samples = 1;
    float shadow = 0.0;
    for (int s=0; s<samples; s++) {
        vec3 offset = bluerand4(blue_noise, int(mod(s + samples*noise_seed, 256))).xyz;
        shadow += subTextureCubeShadow(
            shadowmaps,
            idx,
            light.base.location.xyz,
            surface.position,
            surface.normal,
            offset,
            light.base.z_near,
            light.base.distance
        );
    }
    shadow /= samples;
    diffuse *= shadow;
    specular *= shadow;
}
#endif