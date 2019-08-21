
#define SUNLIGHT_SIZE 30.0

mat4 bias = mat4(0.5, 0.0, 0.0, 0.5,
                 0.0, 0.5, 0.0, 0.5,
                 0.0, 0.0, 1.0, 0.0,
                 0.0, 0.0, 0.0, 1.0);
mat4 bias2 = mat4(0.5, 0.0, 0.0, 0.5,
                 0.0, 0.5, 0.0, 0.5,
                 0.0, 0.0, 0.5, 0.5,
                 0.0, 0.0, 0.0, 1.0);
                 
mat4 perspective(float w,
                float h,
                float znear,
                float zfar,
                float angle)
{
    mat4 result;
    angle = 1.0/tan(angle/2.0/180.0*3.1415926535);
    result[0] = vec4(angle/w*h,0.0,0.0,0.0);
    result[1] = vec4(0.0,angle,0.0,0.0);
    result[2] = vec4(0.0,0.0,-(znear+zfar)/(zfar-znear),-2.0*znear*zfar/(zfar-znear));
    result[3] = vec4(0.0,0.0,-1.0,0.0);
    return (result);
}

mat4 parallel(float size,
              float znear,
              float zfar)
{
    mat4 result;
    result[0] = vec4(2.0/size,	0.0,		0.0,	0.0);
    result[1] = vec4(0.0,	2.0/size,	0.0,	0.0);
    result[2] = vec4(0.0,	0.0,		-2.0/(zfar-znear),(-zfar-znear)/(zfar-znear));
    result[3] = vec4(0.0,	0.0,		0.0,	1.0);
    return result;
}

vec3 transform_ortho(vec4 position,
		              float znear,
		              float zfar)
{
	return vec3(position.xy/20.0*0.5+0.5, (position.z-zfar)/(zfar-znear));
}

mat3 rotationFromMat4(mat4 mat)
{
  return mat3(mat[0].xyz,mat[1].xyz,mat[2].xyz);
}

vec3 globalPosition(mat4 mat)
{
  mat3 rotation = transpose(rotationFromMat4(mat));
  return transpose(mat)[3].xyz * rotation;
}

mat4 interpolaten(mat4 matA,mat4 matB,float coeff){
  mat3 rotA = rotationFromMat4(matA);
  mat3 rotB = rotationFromMat4(matB);
  mat3 rotC = transpose(rotA*(1.0-coeff) + rotB*coeff);
  vec3 posA = globalPosition(matA);
  vec3 posB = globalPosition(matB);
  vec3 posC = mix(posA,posB,coeff);
  rotC[0] /= length(rotC[0]);
  rotC[1] /= length(rotC[1]);
  rotC[2] /= length(rotC[2]);
  rotC = transpose(rotC);
  posC *= rotC;
  return mat4(vec4(rotC[0],posC.x),
              vec4(rotC[1],posC.y),
              vec4(rotC[2],posC.z),
              vec4(0,0,0,  1.0));
}

mat4 interpolate(mat4 mat0,mat4 mat1,float coeff){
  mat4 diff;
  mat4 res;
  for (int i=0;i<4;i++)
    diff[i] = (mat1[i]-mat0[i])*coeff;
  res = transpose(mat0+diff);
  for (int i=0;i<3;i++)
    res[i].xyz=normalize(res[i].xyz);
  res[3].w=1.0;
  return transpose(res);
  //return mat0 + (mat1-mat0)*coeff;
}
