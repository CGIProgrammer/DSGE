#ifndef FXAA_H
#define FXAA_H
    
#define FXAA_SPAN_MAX (8.0)
#define FXAA_REDUCE_MUL (1.0/8.0)
#define FXAA_REDUCE_MIN (1.0/128.0)

vec4 fxaa(sampler2D original, ivec2 pc, vec2 uv) {
    vec4 fragColor;
    
    vec2 offset = 1.0/textureSize(original, 0);
    
    vec3 nw = texelFetch(original, pc + ivec2(-1, -1), 0).rgb;
    vec3 ne = texelFetch(original, pc + ivec2( 1, -1), 0).rgb;
    vec3 sw = texelFetch(original, pc + ivec2(-1,  1), 0).rgb;
    vec3 se = texelFetch(original, pc + ivec2( 1,  1), 0).rgb;
    vec3 m  = texelFetch(original, pc, 0).rgb;

    vec3 luma = vec3(0.299, 0.587, 0.114);
    float lumaNW = dot(nw, luma);
    float lumaNE = dot(ne, luma);
    float lumaSW = dot(sw, luma);
    float lumaSE = dot(se, luma);
    float lumaM  = dot(m,  luma);

    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    vec2 dir = vec2(
        -((lumaNW + lumaNE) - (lumaSW + lumaSE)),
        ((lumaNW + lumaSW) - (lumaNE + lumaSE)));

    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    dir = min(vec2(FXAA_SPAN_MAX), max(vec2(-FXAA_SPAN_MAX), dir * rcpDirMin)) * offset;

    vec3 rgbA = 
        0.5 * (texture(original, uv + dir * (1.0 / 3.0 - 0.5)).xyz
        + texture(original, uv + dir * (2.0 / 3.0 - 0.5)).xyz);
    
    vec3 rgbB =
        rgbA * 0.5 + 0.25 * (texture(original, uv + dir * -0.5).xyz
        + texture(original, uv + dir * 0.5).xyz);
        
    float lumaB = dot(rgbB, luma);
    if (lumaB < lumaMin || lumaB > lumaMax) {
        fragColor = vec4(rgbA, 1.0);
    } else {
        fragColor = vec4(rgbB, 1.0);
    }
    return fragColor;
}

vec4 fxaa(sampler2D original, ivec2 pc, vec2 uv, float mc) {
    vec4 sam = texelFetch(original, pc, 0);
    if (mc > 0.001) {
        return mix(
            sam,
            fxaa(original, pc, uv),
            clamp(mc, 0.0, 1.0)
        );
    } else {
        return sam;
    }
}

#endif