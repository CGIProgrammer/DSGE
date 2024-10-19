#ifndef BILATERAL_H
#define BILATERAL_H

#include "../depth_packing.h"

const float defaultSigmaS = 8.0;

#define SIGMA 8
#define KERNEL_ARRAY_SIZE 13
const float kernel_weights[13] = float[](
   0.0024102853168102537, 0.002448241787590149, 0.002467443585915106, 0.002448241787590149, 0.0024102853168102537, 0.002467443585915106,
   0.0024867959858108648, 0.002467443585915106, 0.0024102853168102537, 0.002448241787590149, 0.002467443585915106, 0.002448241787590149,
   0.0024102853168102537
);
const ivec2 kernel_offsets[13] = ivec2[](
   ivec2(-3, -1), ivec2(-2, -2), ivec2(-2, -1), ivec2(-2, 0), ivec2(-1, -3), ivec2(-1, -2),
   ivec2(-1, -1), ivec2(-1, 0), ivec2(-1, 1), ivec2(0, -2), ivec2(0, -1), ivec2(0, 0),
   ivec2(1, -1)
);

vec3 bilateralFilter(sampler2D scene, ivec2 coord, ivec2 kernel_offset, vec3 centerNormal, float centerDepth, float sigmaS, float z_near, float z_far) {
    vec3 centerColor = texelFetch(scene, coord, 0).rgb;
    vec3 centerMasks = texelFetch(gMasks, coord, 0).rgb;
    vec3 sumColor = vec3(0.0);
    float sumWeight = 0.0;
    #ifdef SUPER_RESOLUTION
    const int scale = 1;
    #else
    const int scale = 2;
    #endif
    //centerColor = LinearToSRGB(ACESFilm(centerColor));
    for (int i=0; i<KERNEL_ARRAY_SIZE; i++) {
        ivec2 offset = kernel_offsets[i] * scale + kernel_offset;
        ivec2 neighborCoord = coord + offset;
        vec3 neighborColor = texelFetch(scene, neighborCoord, 0).rgb;
        vec3 neighborNormal = texelFetch(gNormals, neighborCoord, 0).rgb;
        vec3 neighborMasks = texelFetch(gMasks, neighborCoord, 0).rgb;
        float neighborDepth = texelFetch(gDepth, neighborCoord, 0).r;
        float colorDist = distance(centerColor, neighborColor) * 10.0;
        float normalDist = (1.0 - max(dot(centerNormal, neighborNormal), 0.0)) * 10.0;
        float depthDist = (distance(centerDepth, neighborDepth) * 10000.0 + 0.5);

        float spatialWeight = kernel_weights[i];
        float rangeWeight = exp(-colorDist * normalDist * depthDist);
        //float rangeWeight = 1.0 / (colorDist * normalDist * depthDist + 1.0);

        float weight = spatialWeight * rangeWeight;
        sumColor += neighborColor * weight;
        sumWeight += weight;
    }

    return sumColor / sumWeight;
}

#endif