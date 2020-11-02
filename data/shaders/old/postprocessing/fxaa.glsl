#include "data/shaders/functions/extensions.glsl"
#include "data/shaders/functions/projections.glsl"

uniform sampler2D filtered,original,gVectors, gMasks;

#ifndef FXAA_REDUCE_MIN
    #define FXAA_REDUCE_MIN   (1.0/ 128.0)
#endif
#ifndef FXAA_REDUCE_MUL
    #define FXAA_REDUCE_MUL   (1.0 / 8.0)
#endif
#ifndef FXAA_SPAN_MAX
    #define FXAA_SPAN_MAX     8.0
#endif

//optimized version for mobile, where dependent 
//texture reads can be a bottleneck
vec4 fxaa(sampler2D tex, vec2 fragCoord, vec2 resolution) {
    vec4 color;
    mediump vec2 inverseVP = vec2(1.0 / resolution.x, 1.0 / resolution.y);
//#define TOFFSET
#ifndef TOFFSET
    vec3 rgbNW = texture(tex, fragCoord + vec2(-1.0,  1.0)/resolution).rgb;
    vec3 rgbNE = texture(tex, fragCoord + vec2( 1.0,  1.0)/resolution).rgb;
    vec3 rgbSW = texture(tex, fragCoord + vec2(-1.0, -1.0)/resolution).rgb;
    vec3 rgbSE = texture(tex, fragCoord + vec2( 1.0, -1.0)/resolution).rgb;
#else
    vec3 rgbNW = textureOffset(tex, fragCoord, ivec2(-1, 1)).rgb;
    vec3 rgbNE = textureOffset(tex, fragCoord, ivec2(1, 1)).rgb;
    vec3 rgbSW = textureOffset(tex, fragCoord, ivec2(-1, -1)).rgb;
    vec3 rgbSE = textureOffset(tex, fragCoord, ivec2(1, -1)).rgb;
#endif

    vec4 texColor = texture2D(tex, fragCoord);
    fragCoord.x*=resolution.x;
    fragCoord.y*=resolution.y;
    vec3 rgbM  = texColor.xyz;
    
    vec3 luma = vec3(0.299, 0.587, 0.114);
    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM,  luma);
    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    
    mediump vec2 dir;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));
    
    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) *
                          (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
              max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
              dir * rcpDirMin)) * inverseVP;
    
    vec3 rgbA = 0.5 * (
        texture(tex, fragCoord * inverseVP + dir * (1.0 / 3.0 - 0.5)).xyz +
        texture(tex, fragCoord * inverseVP + dir * (2.0 / 3.0 - 0.5)).xyz);
    vec3 rgbB = rgbA * 0.5 + 0.25 * (
        texture(tex, fragCoord * inverseVP + dir * -0.5).xyz +
        texture(tex, fragCoord * inverseVP + dir * 0.5).xyz);

    float lumaB = dot(rgbB, luma);
    if ((lumaB < lumaMin) || (lumaB > lumaMax))
        color = vec4(rgbA, texColor.a);
    else
        color = vec4(rgbB, texColor.a);
    return color;
}

in vec2 tex_map;

uniform float width,height;

void main()
{
    /*vec3 samp = vec3(0.0);
    int counter = 0;
    for (int i=-2; i<=2; i++)
    for (int j=-2; j<=2; j++)
    {
        samp += texture(filtered, tex_map + vec2(i,  j)/vec2(width, height)).rgb;
        counter++;
    }

    samp /= counter;

    vec3 s = texture(filtered, tex_map).rgb;

    samp = mix(s, samp, abs(length(samp-s)));*/

    fragColor = fxaa(filtered, tex_map, vec2(width,height));
    //fragColor.rgb = vec3(texture(gMasks, tex_map).b);
}