#include "../tonemaping.glsl"
#include "../filters/bilateral.h"
#include "../depth_packing.h"

vec4 denoise(sampler2D scene, ivec2 tex_map, vec2 camera_zrange)
{
    vec4 fragColor = vec4(0.0);
    float d = linearize_depth(texelFetch(gDepth, tex_map, 0).r, camera_zrange.x, camera_zrange.y);
    #ifdef SUPER_RESOLUTION
    int scale = 2;
    #else
    int scale = 2;
    #endif
    int step = 1 * scale;
    int radius = 2 * scale;
    float factCount = 1.0;
    fragColor.rgb = texelFetch(scene, tex_map, 0).rgb;
    vec3 nor = texelFetch(gNormals, tex_map, 0).rgb;
    float r0 = texelFetch(gMasks, tex_map, 0).g;

    for (int i=-radius; i<=radius; i+=step)
    for (int j=-radius; j<=radius; j+=step)
    {
      ivec2 offset = ivec2(j, i);
      if (texelFetch(gAlbedo, tex_map + offset, 0).a<0.5 || i==j)
      {
        continue;
      }
      float dd = linearize_depth(texelFetch(gDepth, tex_map+offset, 0).r, camera_zrange.x, camera_zrange.y);
      vec3 nor_s = texelFetch(gNormals, tex_map + offset, 0).rgb;
      float roughness = min(abs(texelFetch(gMasks, tex_map + offset, 0).g - r0) * 10.0, 1.0);
      float metallic = min(abs(texelFetch(gMasks, tex_map + offset, 0).b) * 10.0, 1.0);
      float dist = abs(dd-d);
      float coeff = abs(dot(nor, nor_s));
      coeff = pow(coeff, 19.0) * (1.0-roughness);// * (1.0-metallic);
      //coeff*= float(dist<0.05);
      coeff/= 1.0 + 15.0 * dist*dist;
      factCount += coeff;
      vec3 sample1 = texelFetch(scene, tex_map+offset, 0).rgb;
      fragColor.rgb += clamp(sample1, 0.0, 5.0) * coeff;
    }
    fragColor.rgb = fragColor.rgb / factCount;
    return fragColor;
}

ivec2 neighbour_offsets[8] = ivec2[8](
  ivec2(-1,  1), ivec2(0,  1), ivec2(1, 1),
  ivec2(-1,  0),               ivec2(1, 0),
  ivec2(-1, -1), ivec2(0, -1), ivec2(1, 1)
);

void main()
{
    vec2 camera_zrange = depth_range(camera.projection);
    ivec2 pixelCoord = ivec2(pixelCoord);
    float roughness = texelFetch(gMasks, pixelCoord, 0).g;
    vec3 diff_coeff = texelFetch(gAlbedo, pixelCoord, 0).rgb;
    float spec_coeff = max(texelFetch(gMasks, pixelCoord, 0).r, texelFetch(gMasks, pixelCoord, 0).b) + 1.0 / 256.0;
    vec3 diff = texelFetch(diffuse_input, pixelCoord, 0).rgb;
    vec3 spec = texelFetch(specular_input, pixelCoord, 0).rgb;

    vec3 centerNormal = texelFetch(gNormals, pixelCoord, 0).rgb;
    float centerDepth = texelFetch(gDepth, pixelCoord, 0).r;
    if (centerDepth == 1.0) {
        composition_out = vec4(1.0);
        return;
    }
    float sigmaS = (roughness * defaultSigmaS + 1.0);
    ivec2 offset = ivec2(0); //ivec2(timer.frame&1, (timer.frame>>1)&1);
    diff = denoise(diffuse_input, pixelCoord, camera_zrange).rgb;
    // spec = denoise(specular_input, pixelCoord, camera_zrange).rgb;
    
    // diff = bilateralFilter(diffuse_input, pixelCoord, offset, centerNormal, centerDepth, defaultSigmaS, camera_zrange.r, camera_zrange.g);
    // spec = bilateralFilter(specular_input, pixelCoord, offset, centerNormal, centerDepth, sigmaS, camera_zrange.r, camera_zrange.g);
    vec3 neighbours_sum = vec3(0.0);
    int neighbours_cnt = 0;
    // if (spec == vec3(0.0)) {
    //   for (int i=0; i<8; i++) {
    //     vec3 nei = texelFetch(specular_input, pixelCoord + neighbour_offsets[i], 0).rgb;
    //     if (nei != vec3(0.0)) {
    //       neighbours_sum += nei;
    //       neighbours_cnt++;
    //     }
    //   }
    //   if (neighbours_cnt > 0) {
    //     spec = neighbours_sum / neighbours_cnt;
    //   }
    // }

    composition_out.rgb = diff*diff_coeff + spec*spec_coeff;
    composition_out.rgb = LinearToSRGB(ACESFilm(composition_out.rgb));
    /*if (composition_out.rgb == vec3(0.0)) {
        composition_out.rgb = vec3(1.0, 0.0, 1.0);
    }*/
    //composition_out.rgb = composition_out.rgb / (composition_out.rgb + 1.0);
    composition_out.a = 1.0;
}