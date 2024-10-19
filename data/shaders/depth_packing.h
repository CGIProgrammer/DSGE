#ifndef DEPTH_PACKING_GLSL
#define DEPTH_PACKING_GLSL

vec2 depth_range(mat4 proj)
{
  float i = proj[2][2];
  float k = proj[3][2];
  float near = k/(i - 1.0);
  float far = k/(i + 1.0);
  return vec2(near, far);
}
vec2 near_and_far(mat4 proj)
{
  return depth_range(proj);
}
vec2 depth_range_inv(mat4 proj)
{
  float k = proj[3][3];
  float i = proj[2][3];
  return vec2(-1.0/(i - k), 1.0/(i+k));
}

float normalize_depth(float z_e, float zNear, float zFar)
{
    return (abs(z_e) - zNear) / (zFar - zNear);
}
float unpack_depth(float z_e, float zNear, float zFar)
{
    return (abs(z_e) - zNear) / (zFar - zNear);
}

float linearize_depth(float z_b, float zNear, float zFar)
{
    return 2.0 * zNear * zFar / (zFar + zNear - z_b * (zFar - zNear));
}

vec4 gPosition(float linear_depth, vec2 screenCoord, mat4 projInv, mat4 modelview)
{
  vec4 pos = vec4((screenCoord*2.0-1.0)*linear_depth, 0.0, 1.0);
  pos = projInv * pos;
  pos.zw = vec2(-linear_depth,1.0);
  pos = modelview * pos;
  pos.w = 1.0;
  return pos;
}

vec3 gRenderNormal(sampler2D normalMap4, vec2 screenCoord)
{
  //return texture(normalMap4, screenCoord).rgb;
  return texture(normalMap4, screenCoord).rgb*2.0 - 1.0;
}

#endif