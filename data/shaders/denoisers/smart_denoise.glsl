#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gLighting, gAlbedo, gSpace, gOutput, gMasks;
uniform float width, height;
uniform int gFilterPass;

#define INV_SQRT_OF_2PI 0.39894228040143267793994605993439  // 1.0/SQRT_OF_2PI
#define INV_PI 0.31830988618379067153776752674503

vec4 smartDeNoise(sampler2D tex, vec2 uv, float sigma, float kSigma, float threshold)
{
    float radius = round(kSigma*sigma);
    float radQ = radius * radius;
    
    float invSigmaQx2 = .5 / (sigma * sigma);      // 1.0 / (sigma^2 * 2.0)
    float invSigmaQx2PI = INV_PI * invSigmaQx2;    // 1.0 / (sqrt(PI) * sigma)
    
    float invThresholdSqx2 = .5 / (threshold * threshold);     // 1.0 / (sigma^2 * 2.0)
    float invThresholdSqrt2PI = INV_SQRT_OF_2PI / threshold;   // 1.0 / (sqrt(2*PI) * sigma)
    
    vec4 centrPx = texture(tex,uv);
    
    float zBuff = 0.0;
    vec4 aBuff = vec4(0.0);
    vec2 size = vec2(textureSize(tex, 0));
    float nw = 0.0;

    vec3 nor = gRenderNormal(gSpace, uv);
    float dd0 = gRenderDepth(gSpace, uv);

    for(float x=-radius; x <= radius; x++) {
        float pt = sqrt(radQ-x*x);  // pt = yRadius: have circular trend
        for(float y=-pt; y <= pt; y++) {
            vec2 d = vec2(x,y);
            float blurFactor = exp( -dot(d , d) * invSigmaQx2 ) * invSigmaQx2PI; 
            d /= size;

            float dd = gRenderDepth(gSpace, uv + d);
            float dist = abs(dd0 - dd);
            vec3 nor_s = gRenderNormal(gSpace, uv + d);
            float coeff = abs(dot(nor, nor_s));
            coeff = pow(coeff, 8.0);
            coeff*= float(dist<0.025);
            
            vec4 walkPx =  texture(tex,uv+d);

            vec4 dC = walkPx-centrPx;
            float deltaFactor = exp( -dot(dC, dC) * invThresholdSqx2) * invThresholdSqrt2PI * blurFactor;
            zBuff += deltaFactor*coeff;
            aBuff += deltaFactor*walkPx*coeff;
        }
    }
    return aBuff/zBuff;
}

void main() {
    //vec2 pass = float(gFilterPass/2 == (gFilterPass+1)/2) / vec2(width, height);
    if (texture(gAlbedo, tex_map).a < 0.5)
    {
      fragColor = texture(gOutput, tex_map);
      return;
    }
    fragColor = smartDeNoise(gOutput, tex_map, 5.0, 2.0, 0.25);
}
