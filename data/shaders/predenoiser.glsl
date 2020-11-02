#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gLSpecular, gAlbedo, gSpace, gOutput, gMasks;
uniform samplerCube cubemap;
uniform float width, height;
uniform int gFilterPass;
uniform mat4 vCameraProjectionInv, vCameraTransform;

vec2[] offsets = vec2[](
    vec2(-1.0,-1.0),vec2( 0.0,-1.0),vec2( 1.0,-1.0),
    vec2(-1.0, 0.0),                vec2( 1.0, 0.0),
    vec2(-1.0, 1.0),vec2( 0.0, 1.0),vec2( 1.0, 1.0)
);

#define LEFT(sampler, uv)         max(texture(sampler, uv + offsets[3]/vec2(width, height)), 0.0)
#define RIGHT(sampler, uv)        max(texture(sampler, uv + offsets[4]/vec2(width, height)), 0.0)
#define TOP(sampler, uv)          max(texture(sampler, uv + offsets[1]/vec2(width, height)), 0.0)
#define BOTTOM(sampler, uv)       max(texture(sampler, uv + offsets[6]/vec2(width, height)), 0.0)
#define TOPLEFT(sampler, uv)      max(texture(sampler, uv + offsets[0]/vec2(width, height)), 0.0)
#define TOPRIGHT(sampler, uv)     max(texture(sampler, uv + offsets[2]/vec2(width, height)), 0.0)
#define BOTTOMLEFT(sampler, uv)   max(texture(sampler, uv + offsets[5]/vec2(width, height)), 0.0)
#define BOTTOMRIGHT(sampler, uv)  max(texture(sampler, uv + offsets[7]/vec2(width, height)), 0.0)


void main() {
    if (texture(gAlbedo, tex_map).a < 0.5)
    {
      fragColor = texture(gOutput, tex_map);
      return;
    }
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
        fragColor.rgb = max(result, vec3(0.0));
    } else {
        fragColor.rgb = max(values[0], vec3(0.0));
    }
    fragColor.a = 1.0;
}