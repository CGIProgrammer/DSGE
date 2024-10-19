//layout (origin_upper_left) in vec4 gl_FragCoord;
precision highp float;

#include "shadowmap.h"
#include "pbr.fs.glsl"
#include "../depth_packing.h"
#include "../constants.h"
#include "../tonemaping.glsl"


void main()
{
    vec2 camera_zrange = depth_range(camera.projection);
    ivec2 pixelCoord = ivec2(pixelCoord);
    
    float linear_depth = linearize_depth(texelFetch(gDepth, pixelCoord, 0).r, camera_zrange.x, camera_zrange.y);
    
    diffuse_out = vec4(0.0);
    specular_out = vec4(0.0);

    if (texelFetch(gDepth, pixelCoord, 0).r==1.0) {
        linear_depth = 1e14;
        return;
    }

    vec3 albedo = texelFetch(gAlbedo, pixelCoord, 0).rgb;
    vec3 normals = normalize(texelFetch(gNormals, pixelCoord, 0).rgb);

    vec3 position = gPosition(linear_depth, fragCoord, camera.projection_inverted, camera.transform).xyz;
    vec4 masks = texelFetch(gMasks, pixelCoord, 0);
    PBRSurface surface;
    surface.position = position;
    surface.albedo = albedo;
    surface.normal = normals;
    surface.specular = masks.r;
    surface.roughness = masks.g;
    surface.metalness = masks.b;

    vec3 ambient_light = vec3(0.0);
    vec3 lighting = ambient_light;
    vec3 nV = normalize(camera.transform[3].xyz - position);
    vec3 l_diffuse = vec3(0), l_specular = vec3(0);
    vec3 summ_diffuse = vec3(0), summ_specular = vec3(0);

    for (int i=0; i<lights_count.spotlights; i++) {
        l_diffuse = vec3(0);
        l_specular = vec3(0);
        Spotlight spl = ppSpotlights[i];
        spl.base.power *= 0.01;
        if (ppSpotlights[i].base.shadowmap_index >= 0) {
            spotlight(spl, spot_shadowmaps, surface, nV, int(mod(timer.frame, 256)), l_diffuse, l_specular);
        } else {
            spotlight(spl, surface, nV, l_diffuse, l_specular);
        }
        summ_diffuse += l_diffuse;
        summ_specular += l_specular;
    }

    for (int i=0; i<lights_count.sun_lights; i++) {
        l_diffuse = vec3(0);
        l_specular = vec3(0);
        if (ppSunlights[i].base.shadowmap_index >= 0) {
            sunlight(ppSunlights[i], sun_shadowmaps, surface, nV, int(mod(timer.frame, 256)), l_diffuse, l_specular);
        } else {
            sunlight(ppSunlights[i], surface, nV, l_diffuse, l_specular);
        }
        summ_diffuse += l_diffuse * 0.2;
        summ_specular += l_specular * 0.4;
    }

    for (int i=0; i<lights_count.point_lights; i++) {
        l_diffuse = vec3(0);
        l_specular = vec3(0);
        PointLight pl = ppPointlights[i];
        pl.base.power *= 0.01;
        if (pl.base.shadowmap_index > -1) {
            pointlight(pl, point_shadowmaps, surface, nV, int(mod(timer.frame, 256)), l_diffuse, l_specular);
        } else {
            pointlight(pl, surface, nV, l_diffuse, l_specular);
        }
        summ_diffuse += l_diffuse;
        summ_specular += l_specular;
    }
    
    diffuse_out.rgb = max(summ_diffuse, 0.0);
    specular_out.rgb = max(summ_specular, 0.0);
    //swapchain_out.rgb = LinearToSRGB(ACESFilm(swapchain_out.rgb));
}