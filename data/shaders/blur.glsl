#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gAlbedo, gSpace, gOutput, gMasks;
uniform vec2 gResolution;

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

vec4 mean_curva(sampler2D image, vec2 center_crd)
{
    vec4 center = max(texture(image, center_crd), 0.0);
    vec4 dx  = RIGHT(image, center_crd) - LEFT(image, center_crd);
    vec4 dy  = BOTTOM(image, center_crd) - TOP(image, center_crd);
    vec4 result = center /* 0.8 + 
        (LEFT(image, center_crd)*0.1 + RIGHT(image, center_crd)*0.1 + 
        BOTTOM(image, center_crd)*0.1 + TOP(image, center_crd)*0.1 + 
        BOTTOMLEFT(image, center_crd)*0.0707 + TOPLEFT(image, center_crd)*0.0707 + 
        BOTTOMRIGHT(image, center_crd)*0.0707 + TOPRIGHT(image, center_crd)*0.0707) / 0.6828 * 0.2*/;

    vec4 dx2 = dx*dx;
    vec4 dy2 = dy*dy;
    vec4 dxx = RIGHT(image, center_crd) + LEFT(image, center_crd) - 2. * center;
    vec4 dyy = BOTTOM(image, center_crd) + TOP(image, center_crd) - 2. * center;
    vec4 dxy = 0.25 * (BOTTOMRIGHT(image, center_crd) - TOPRIGHT(image, center_crd) - BOTTOMLEFT(image, center_crd) + TOPLEFT(image, center_crd));
    vec4 n = dx2 * dyy + dy2 * dxx - 2. * dx * dy * dxy;
    vec4 d = pow(dx2 + dy2, vec4(3.0/2.0));
    vec4 mean_curvature = n / d;
    float roughness = texture(gMasks, tex_map).g;
    vec4 mag = sqrt(dx*dx + dy*dy);
    if (mag.r>0.0001 && mag.g>0.0001 && mag.b>0.0001) result += 0.25 * mag * mean_curvature;
    //if (mag.r>0.0001) result.r += 0.25 * mag.r * mean_curvature.r;
    //if (mag.g>0.0001) result.g += 0.25 * mag.g * mean_curvature.g;
    //if (mag.b>0.0001) result.b += 0.25 * mag.b * mean_curvature.b;
    //if (mag.a>0.0001) result.a += 0.25 * mag.b * mean_curvature.a;
    return result;
}

void main() {
    float alpha = texture(gAlbedo, tex_map).a;
    if (alpha < 0.5)
    {
      fragColor = texture(gOutput, tex_map);
      return;
    }
    fragColor.rgb = mean_curva(gOutput, tex_map).rgb;
    fragColor.a = texture(gOutput, tex_map).a;
}