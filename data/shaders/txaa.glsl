#include "data/shaders/functions/extensions.glsl"

uniform sampler2D gOutput, gAlbedo, gMasks;
uniform sampler2D gAccumulator;
uniform sampler2D gVectors;
uniform sampler2D gSpace;
uniform vec2 gResolution;
uniform int gDitherIteration;
uniform vec2 gDither;
//uniform mat4 vCameraTransformPrev, vCameraTransform, vCameraProjectionInv, vCameraProjection, vCameraTransformInv;

input vec2 tex_map;
input vec2 screen_map;
//uniform vec2 resolution;

vec2 dither[] = {
    vec2(-0.98732421,  0.859431  ),
    vec2(-0.65821248, -0.37457928),
    vec2( 0.25716888,  0.1231111 ),
    vec2( 0.43520789, -0.58960568),
    vec2( 0.18049204,  0.1479625 ),
    vec2( 0.47397861,  0.66341217),
    vec2( 0.10755945, -0.68278827),
    vec2( 0.2788744 , -0.62489427),
    vec2(-0.71277244,  0.25320682),
    vec2(-0.3370736 ,  0.28901948),
    vec2( 0.36766457, -0.10139287),
    vec2( 0.29690737, -0.79951376),
    vec2(-0.68917806,  0.4233432 ),
    vec2( 0.15096014,  0.85329404),
    vec2(-0.20626524, -0.60187284),
    vec2( 0.34081387, -0.86511024)
};

float edge_detection(sampler2D buff, float r)
{
    float cd = unpack_depth(texture(buff, tex_map).a);
    float edge = 0.0;
    for (float i=-r; i<=r; i++) {
        for (float j=-r; j<=r; j++) {
            edge += abs(unpack_depth(texture(buff, tex_map + vec2(i,j)/gResolution).a) - cd);
        }
    }
    return float(edge / pow(2.0*r+1.0, 2.0)>0.2);
}

void main()
{
    //vec4 cpos = gPosition(gSpace, tex_map, vCameraProjectionInv, vCameraTransform) * vCameraTransformInv * vCameraProjection;
    //vec4 ppos = gPosition(gVectors, tex_map, vCameraProjectionInv, vCameraTransformPrev) * vCameraTransformInv * vCameraProjection;
    //vec2 vel = (ppos.xy/ppos.w - cpos.xy/cpos.w) * 0.5;
    //fragColor = texture(gOutput, tex_map);
    //return;
    vec2 crd = (fract(screen_map * 0.5) - 0.5);
    float cell;
    vec2 crd1 = vec2(float(crd.x>=0.0), float(crd.y>=0.0));
    vec2 crd2 = vec2(float(-gDither.x>=0.0), float(gDither.y>=0.0));
    float smooth_mask = clamp(1.0-length(crd-crd2), 0.0, 1.0);
    cell = float(crd1.x==crd2.x && crd1.y==crd2.y);
    crd2 = (crd2-0.5)*2.0 / gResolution;

    vec2 vel = texture(gVectors, tex_map).xy;
    float alpha = 1.0 - texture(gAlbedo, tex_map).a;
    
    float cd = texture(gSpace, tex_map).a;
    vec4 prev = texture(gAccumulator, tex_map-vel);
    vec4 curr = vec4(texture(gOutput, tex_map - crd2*0.0).rgb, cd);
    float edge = 1.0 - edge_detection(gAccumulator, 2.0);
    float d = abs(cd - prev.a);// * 75.0;
    const float r = 2.0;
    for (float i=-r; i<=r; i++)
    {
        for (float j=-r; j<=r; j++)
        {
            float depth = texture(gAccumulator, tex_map-vel + vec2(i,j)/gResolution*1.0).a;
            d = min(d, abs(cd - depth));
            //d += abs(cd - depth);
        }
    }
    d *= 200.0;
    d += max(length(vel)-2.0/length(gResolution), 0.0) * (1.0-texture(gMasks, tex_map).g) * 300.0;
    //cell = mix(cell, edge, clamp(d, 0.0, 1.0));
    //d = edge;
    //d = abs(unpack_depth(cd) - unpack_depth(prev.a)) * 50.0 * );
    cell = 1.0;
    crd = tex_map-vel;
    if (crd.x>1.0 || crd.y>1.0 || crd.x<0.0 || crd.y<0.0) {
        d = 1.0;
    }
    vec2 ires = 1.0 / gResolution;
    float coeff = clamp(alpha + d, 0.1, 1.0);
    
    fragColor = mix(prev, curr, coeff*cell);
    //fragColor.a = mix(prev, curr, coeff).a;
    if (isnan(fragColor.r) || isnan(fragColor.g) || isnan(fragColor.b) || isnan(fragColor.a)) {
        fragColor = curr;
    }
    //fragColor.rgb = vec3(1.0-d);// * vec3(checker[int(mod(gDitherIteration, 4))]);
    //fragColor.xyz = vec3(texture(gAccumulator, tex_map) + 0.002);
} 
