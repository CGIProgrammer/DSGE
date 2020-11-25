
//#define TEXTURE_RANDOM
#ifdef TEXTURE_RANDOM
const float noise_tile_resolution = 64.0;
const float noise_tiles = 64.0;
uniform sampler2D gNoise;

float blueRand1(vec2 crd,int num)
{
    float loop = float(noise_tile_resolution);
    float x = (fract(crd.x*gResolution.x / loop) + float(num)) / noise_tiles;
    float y = crd.y*gResolution.y / loop;
    return texture(gNoise, vec2(x, y)).r * 2.0 - 1.0;
}
vec2 blueRand2(vec2 crd,int num)
{
    float loop = float(noise_tile_resolution);
    float x = (fract(crd.x*gResolution.x / loop) + float(num)) / noise_tiles;
    float y = crd.y*gResolution.y / loop;
    return texture(gNoise, vec2(x, y)).rg * 2.0 - 1.0;
} 
vec3 blueRand3(vec2 crd,int num)
{
    float loop = float(noise_tile_resolution);
    float x = (fract(crd.x*gResolution.x / loop) + float(num)) / noise_tiles;
    float y = crd.y*gResolution.y / loop;
    return texture(gNoise, vec2(x, y)).rgb * 2.0 - 1.0;
} 
#else
float blueRand1(vec2 crd, int num)
{
  return fract(sin(dot(crd.xy*float(num+1) ,vec2(12.9898,78.233))) * 43758.5453) * 2.0 - 1.0;
}
vec2 blueRand2(vec2 crd, int num)
{
  return vec2(blueRand1(crd + vec2(float(num), 0.0), num), blueRand1(crd + vec2(0.0, float(num)), num));
}
vec3 blueRand3(vec2 crd, int num)
{
  return vec3(
    blueRand1(crd + vec2(float(num)),      num),
    blueRand1(crd + vec2(float(num), 0.0), num),
    blueRand1(crd + vec2(0.0, float(num)), num)
  );
}
#endif
