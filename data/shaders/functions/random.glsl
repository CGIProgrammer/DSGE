
float noise_tile_resolution = 8;
 vec3 blueRand3(vec2 crd,int num)
{
  float pix = 1.0 / width;
  return vec3(texture(gNoise,vec3(crd.x*width/noise_tile_resolution, crd.y*height/noise_tile_resolution, num*3)).r,
              texture(gNoise,vec3(crd.x*width/noise_tile_resolution, crd.y*height/noise_tile_resolution, num*3+1)).r,
              texture(gNoise,vec3(crd.x*width/noise_tile_resolution, crd.y*height/noise_tile_resolution, num*3+2)).r);
}
vec2 blueRand2(vec2 crd,int num)
{
  float pix = 1.0 / width;
  return vec2(texture(gNoise,vec3(crd.x*width/noise_tile_resolution, crd.y*height/noise_tile_resolution, num*2)).r,
              texture(gNoise,vec3(crd.x*width/noise_tile_resolution, crd.y*height/noise_tile_resolution, num*2+1)).r);
} 
float blueRand1(vec2 crd,int num)
{
  float pix = 1.0 / width;
  return texture(gNoise,vec3(crd.x*width/noise_tile_resolution, crd.y*height/noise_tile_resolution, num)).r;
} 
