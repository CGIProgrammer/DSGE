#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gLSpecular, gAlbedo, gSpace, gOutput, gMasks;
uniform samplerCube cubemap;
uniform vec2 gResolution;
uniform int gFilterPass;
uniform mat4 vCameraProjectionInv, vCameraTransform;

vec2[] offsets = vec2[](
    vec2(-1.0,-1.0),vec2( 0.0,-1.0),vec2( 1.0,-1.0),
    vec2(-1.0, 0.0),                vec2( 1.0, 0.0),
    vec2(-1.0, 1.0),vec2( 0.0, 1.0),vec2( 1.0, 1.0)
);

#define LEFT(sampler, uv)         max(texture(sampler, uv + offsets[3]/gResolution), 0.0)
#define RIGHT(sampler, uv)        max(texture(sampler, uv + offsets[4]/gResolution), 0.0)
#define TOP(sampler, uv)          max(texture(sampler, uv + offsets[1]/gResolution), 0.0)
#define BOTTOM(sampler, uv)       max(texture(sampler, uv + offsets[6]/gResolution), 0.0)
#define TOPLEFT(sampler, uv)      max(texture(sampler, uv + offsets[0]/gResolution), 0.0)
#define TOPRIGHT(sampler, uv)     max(texture(sampler, uv + offsets[2]/gResolution), 0.0)
#define BOTTOMLEFT(sampler, uv)   max(texture(sampler, uv + offsets[5]/gResolution), 0.0)
#define BOTTOMRIGHT(sampler, uv)  max(texture(sampler, uv + offsets[7]/gResolution), 0.0)

vec3 renderSky(mat4 ori)
{
    vec4 skyboxVector = vec4((tex_map*2.0-1.0)*5.0,0.0,0.0) * vCameraProjectionInv;
    ori = inverse(ori);
    skyboxVector.zw = vec2(-5.0, 1.0);
    skyboxVector *= mat4(vec4(ori[0].x,ori[1].x, ori[2].x, 0.0),
                         vec4(ori[0].y,ori[1].y, ori[2].y, 0.0),
                         vec4(ori[0].z,ori[1].z, ori[2].z, 0.0),
                         vec4(0.0,0.0,0.0,1.0));
    skyboxVector = normalize(skyboxVector);
    return textureCubemap(cubemap, skyboxVector.xyz).rgb;
}

vec3 reinhard_extended(vec3 v, float max_white)
{
    vec3 numerator = v * (1.0f + (v / vec3(max_white * max_white)));
    return numerator / (1.0f + v);
}

void main() {
    vec4 masks = texture(gMasks, tex_map);
    vec3 mDiffuse = texture(gAlbedo, tex_map).rgb;
    float mSpecular = masks.r;
    float mRoughness = masks.g;
    float mMetallic = masks.b;

    vec3 mean = vec3(0.0);
    vec3[] values = vec3[] (
        texture(gOutput, tex_map).rgb,
        LEFT(gOutput, tex_map).rgb,
        RIGHT(gOutput, tex_map).rgb,
        TOP(gOutput, tex_map).rgb,
        BOTTOM(gOutput, tex_map).rgb,
        TOPLEFT(gOutput, tex_map).rgb,
        TOPRIGHT(gOutput, tex_map).rgb,
        BOTTOMLEFT(gOutput, tex_map).rgb,
        BOTTOMRIGHT(gOutput, tex_map).rgb
    );
    for (int i=0; i<9; i++) {
        mean += values[i] / 9.0;
    }

    float diff = length(mean-values[0]);
    int num = 0;
    vec3 result = mean;
    for (int i=1; i<9; i++) {
        float d = length(mean-values[i]);
        if (d>diff) {
            diff = d;
            num = i;
        }
    }
    if (num==0) {
        result -= values[num]/9.0;
        result = result * 9.0 / 8.0;
        fragColor.rgb = result;
    } else {
        fragColor.rgb = values[0];
    }
    
    if (texture(gAlbedo, tex_map).a < 1.0) {
        fragColor = vec4(renderSky(vCameraTransform) * 2.0, 1.0); //renderSky(vCameraTransform);
        //fragColor.rgb *= fragColor.rgb;
    } else {
        fragColor.rgb = fragColor.rgb + texture(gLSpecular, tex_map).rgb;
        fragColor.a = 1.0;
    }
    fragColor.rgb = pow(fragColor.rgb, vec3(0.454545));
    fragColor.rgb = reinhard_extended(fragColor.rgb, 1.5);
    //fragColor.rgb = (exp(2.0 * fragColor.rgb) - 1.0) / (exp(2.0 * fragColor.rgb) + 1.0);
}