#include "../depth_packing.h"

#define FOR(var, start, end, step) for (int var=(start); var<=(end); var+=(step))
#define FOR2D(var_i, var_j, start, end, step) FOR(var_i, start, end, step) FOR(var_j, (start), (end), (step))
#define COMPARE_DEPTH(a, b, eps) (abs((a) - (b)) > eps)
#define LIGHTING_SAMPLE(light_sapler, depth_sampler, lowres_d, coords, offset) \
    if (!COMPARE_DEPTH(texelFetch(depth_sampler, (coords) + (offset)*2, 0).r, lowres_d, 0.001)) { \
        val = texelFetch(light_sapler, (coords)/2 + (offset), 0); \
        result = texelFetch(gAlbedo, pixelCoord, 0) * val; \
        result.rgb = result.rgb / (1.0 + result.rgb); \
        return; \
    }

void main() {
    ivec2 pixelCoord = ivec2(pixelCoord);
    ivec2 lightPixelCoord = pixelCoord / 2;
    ivec2 depthPixelCoord = lightPixelCoord * 2;
    float depth_lowres = texelFetch(gDepth, depthPixelCoord, 0).r;
    float depth_reference = texelFetch(gDepth, pixelCoord, 0).r;
    vec4 val = texelFetch(lighting, lightPixelCoord, 0);
    /*float count = 1.0;
    if (COMPARE_DEPTH(depth_lowres, depth_reference, 0.0001)) {
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2(-1,-1));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2(-1, 0));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2(-1, 1));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2( 0,-1));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2( 0, 1));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2( 1,-1));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2( 1, 0));
        LIGHTING_SAMPLE(lighting, gDepth, depth_lowres, pixelCoord, ivec2( 1, 1));
    }
    val /= count;*/
    result = texelFetch(gAlbedo, pixelCoord, 0) * val;
    result.rgb = result.rgb / (1.0 + result.rgb);
    result.a = 1.0;
}