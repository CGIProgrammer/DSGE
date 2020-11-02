
const float VOXEL_MAP_SIZE = 30.0;
const float VOXEL_SIZE = VOXEL_MAP_SIZE/256.0;
const float PI = 3.14159265359;

float atan2(float x, float y)
{
  if (x>0.0) {
    return atan(y/x);
  }
  if (x<0.0 && y>=0.0) {
    return atan(y/x) + PI;
  }
  if (x<0.0 && y<0.0) {
    return atan(y/x) - PI;
  }
  if (x==0.0 && y>0.0) {
    return PI*0.5;
  }
  if (x==0.0 && y<0.0) {
    return -PI*0.5;
  }
  return 0.0/0.0;
}

#if __VERSION__ < 140
mat2 transpose(mat2 m)
{
  return mat2(
              vec2(m[0].x,m[1].x),
              vec2(m[0].y,m[1].y)
            );
}

mat3 transpose(mat3 m)
{
  return mat3(
              vec3(m[0].x,m[1].x,m[2].x),
              vec3(m[0].y,m[1].y,m[2].y),
              vec3(m[0].z,m[1].z,m[2].z)
            );
}

mat4 transpose(mat4 m)
{
  return mat4(
              vec4(m[0].x,m[1].x,m[2].x,m[3].x),
              vec4(m[0].y,m[1].y,m[2].y,m[3].y),
              vec4(m[0].z,m[1].z,m[2].z,m[3].z),
              vec4(m[0].w,m[1].w,m[2].w,m[3].w)
            );
}
#ifndef VERTEX

#if __VERSION__ < 140
vec4 texture(sampler2D sampler, vec2 coords)
{
  return texture2D(sampler, coords);
}

vec4 texture(samplerCube sam, vec3 coords)
{
  return textureCube(sam, coords);
}

vec4 textureLod(sampler2D sampler, vec2 coords, float lod)
{
#ifdef GL_EXT_shader_texture_lod
    return texture2DLod(sampler, coords, lod);
#else
    return texture2D(sampler, coords);
#endif
}

vec4 textureLod(samplerCube sampler, vec3 coords, float lod)
{
#ifdef GL_EXT_shader_texture_lod
    return textureCubeLod(sampler, coords, lod);
#else
    return textureCube(sampler, coords);
#endif
}

/*#ifdef GL_EXT_texture_array
  #if TEXTURE_RANDOM == 1
  vec4 texture(sampler2DArray sampler, vec3 position)
  {
      return texture2DArray(sampler, position);
  }
  #endif
#else
#error Your videocard does not support texture arrays. So you cannot use SSGI effect.
#endif*/

#endif
#endif
#endif

#if __VERSION__ < 140
float inverse(float m) {
  return 1.0 / m;
}

mat2 inverse(mat2 m) {
  return mat2(m[1][1],-m[0][1],
             -m[1][0], m[0][0]) / (m[0][0]*m[1][1] - m[0][1]*m[1][0]);
}

mat3 inverse(mat3 m) {
  float a00 = m[0][0], a01 = m[0][1], a02 = m[0][2];
  float a10 = m[1][0], a11 = m[1][1], a12 = m[1][2];
  float a20 = m[2][0], a21 = m[2][1], a22 = m[2][2];

  float b01 = a22 * a11 - a12 * a21;
  float b11 = -a22 * a10 + a12 * a20;
  float b21 = a21 * a10 - a11 * a20;

  float det = a00 * b01 + a01 * b11 + a02 * b21;

  return mat3(b01, (-a22 * a01 + a02 * a21), (a12 * a01 - a02 * a11),
              b11, (a22 * a00 - a02 * a20), (-a12 * a00 + a02 * a10),
              b21, (-a21 * a00 + a01 * a20), (a11 * a00 - a01 * a10)) / det;
}

mat4 inverse(mat4 m) {
  float
      a00 = m[0][0], a01 = m[0][1], a02 = m[0][2], a03 = m[0][3],
      a10 = m[1][0], a11 = m[1][1], a12 = m[1][2], a13 = m[1][3],
      a20 = m[2][0], a21 = m[2][1], a22 = m[2][2], a23 = m[2][3],
      a30 = m[3][0], a31 = m[3][1], a32 = m[3][2], a33 = m[3][3],

      b00 = a00 * a11 - a01 * a10,
      b01 = a00 * a12 - a02 * a10,
      b02 = a00 * a13 - a03 * a10,
      b03 = a01 * a12 - a02 * a11,
      b04 = a01 * a13 - a03 * a11,
      b05 = a02 * a13 - a03 * a12,
      b06 = a20 * a31 - a21 * a30,
      b07 = a20 * a32 - a22 * a30,
      b08 = a20 * a33 - a23 * a30,
      b09 = a21 * a32 - a22 * a31,
      b10 = a21 * a33 - a23 * a31,
      b11 = a22 * a33 - a23 * a32,

      det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

  return mat4(
      a11 * b11 - a12 * b10 + a13 * b09,
      a02 * b10 - a01 * b11 - a03 * b09,
      a31 * b05 - a32 * b04 + a33 * b03,
      a22 * b04 - a21 * b05 - a23 * b03,
      a12 * b08 - a10 * b11 - a13 * b07,
      a00 * b11 - a02 * b08 + a03 * b07,
      a32 * b02 - a30 * b05 - a33 * b01,
      a20 * b05 - a22 * b02 + a23 * b01,
      a10 * b10 - a11 * b08 + a13 * b06,
      a01 * b08 - a00 * b10 - a03 * b06,
      a30 * b04 - a31 * b02 + a33 * b00,
      a21 * b02 - a20 * b04 - a23 * b00,
      a11 * b07 - a10 * b09 - a12 * b06,
      a00 * b09 - a01 * b07 + a02 * b06,
      a31 * b01 - a30 * b03 - a32 * b00,
      a20 * b03 - a21 * b01 + a22 * b00) / det;
}
#endif

#ifndef VERTEX
float pack_depth(float dist)
{
    return abs(1.0 / (dist+1.0));
};

float unpack_depth(float dist)
{
    return 1.0 / dist - 1.0;
}

vec4 gPosition(sampler2D normalMap4, vec2 screenCoord, mat4 projInv, mat4 modelview)
{
  float dist = unpack_depth(texture(normalMap4, screenCoord).a);
  vec4 pos = vec4((screenCoord*2.0-1.0)*dist, 0.0, 1.0);
  pos*= projInv;
  pos.zw = vec2(-dist,1.0);
  pos*= modelview;
  pos.w = 1.0;
  return pos;
}

float gRenderDepth(sampler2D normalMap4, vec2 screenCoord)
{
  return unpack_depth(texture(normalMap4, screenCoord).a);
}

vec3 gRenderNormal(sampler2D normalMap4, vec2 screenCoord)
{
  //return texture(normalMap4, screenCoord).rgb;
  return texture(normalMap4, screenCoord).rgb*2.0 - 1.0;
}

vec4 textureCubemap(samplerCube spheremap, vec3 vector)
{
  vec4 s = texture(spheremap, vector.xzy);
  vec4 glare = max(s-0.9, 0.0) * 5.0;
  s.rgb = s.rgb/0.9 + glare.rgb;
  return s;
}

vec4 textureCubemap(samplerCube spheremap, vec3 vector, float level)
{
  vec4 s = textureLod(spheremap, vector.xzy, level);
  vec4 glare = max(s-0.9, 0.0) * 5.0;
  s.rgb = s.rgb/0.9 + glare.rgb;
  return s;
}

vec4 textureSpheremap(sampler2D spheremap, vec3 vector)
{
  vector = normalize(vector * vec3(1.0,-1.0,1.0));
  vec2 result;

  result.y   = 1.0-(vector.z*0.5*0.995+0.5);
  vector.xy /= sqrt(1.0-vector.z*vector.z);
  result.x   = acos(vector.y)/3.1415926535/2.0;
  result.x  *= vector.x<0.0 ? 1.0 : -1.0;
  result.x   = (result.x + 0.5);
  return texture(spheremap, result);
}
#endif

vec3 trueCross(vec3 v1, vec3 v2)
{
  return vec3(
      v1.y*v2.z - v1.z*v2.y,
    -(v1.x*v2.z - v1.z*v2.x),
      v1.x*v2.y - v1.y*v2.x
  );
}

mat4 mat3x4to4(mat3x4 mat)
{
  return mat4(mat[0], mat[1], mat[2], vec4(0.0,0.0,0.0,1.0));
}

mat4 mat3x4to4(mat4 mat)
{
  return mat;
}

mat3 mat4to3(mat4 mat)
{
  return mat3(mat[0].xyz, mat[1].xyz, mat[2].xyz);
}
