/*
 * linalg.h
 *
 *  Created on: 23 дек. 2017 г.
 *      Author: ivan
 */

#ifndef LINALG_H_
#define LINALG_H_

#include <math.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <float.h>

#define CHECK_TYPES
#define VECTOR4 4
#define VECTOR 3
#define VECTOR3 3
#define VECTOR2 3
#define COLOR 3
#define UV 2
#define MATRIX16 16
#define MATRIX 16
#define MATRIX9 9

#define Aij(i,j) ((i<<2)|j)

#define A11 0
#define A12 1
#define A13 2
#define A14 3
#define A21 4
#define A22 5
#define A23 6
#define A24 7
#define A31 8
#define A32 9
#define A33 10
#define A34 11
#define A41 12
#define A42 13
#define A43 14
#define A44 15

#define radians(a) ((a)*0.01745329252)
#define degrees(a) ((a)*57.29577951472)

#define laInverted(mat) _Inverted(mat, __FILE__, __LINE__)
extern float laDet;
/*typedef struct
{
	float a[16];
	uint8_t type;
} laType;*/

typedef struct
{
	double a[16];
	uint8_t type;
} laTypeD;

typedef struct
{
	float a[16];
	uint8_t type;
} laTypeS;

typedef laTypeS laType;


typedef struct
{
  float Xx,Yx,Zx,px;
  float Xy,Yy,Zy,py;
  float Xz,Yz,Zz,pz;
  float Xw,Yw,Zw,pw;
  uint8_t type;
} laMatrix;

typedef struct
{
  float x,y,z,w;
} laVector;

typedef enum
{
	laX = 0,
	laY,
	laZ,
	laXn,
	laYn,
	laZn
} laAxis;

float fiSqrt(float x);

laTypeS laTypeCastToSingle(laType*);
laTypeD laTypeCastToDouble(laType*);
laVector laTypeCastToVector(laType);
laMatrix laTypeCastToMatrix(laType);

extern const laType Identity;

laType Cross(laType,laType);
laType Crossn(laType,laType);
float Dot(laType,laType);
float Dotn(laType a,laType b);
void Normalize(laType* mat);

laType Perspective(float,float,float,float,float);
laType Ortho(float size,float zfar,float znear);
laType RotationX(float);
laType RotationY(float);
laType RotationZ(float);
laType RotationXYZ(float,float,float);
void RotateXYZ(laType*,float,float,float);
void RotateXYZlocal(laType*,float,float,float);
void RotateXYZglobal(laType*,float,float,float);
void Translatel(laType* mat,float x,float y,float z);
void Translateg(laType* mat,float x,float y,float z);
void Translate(laType* mat,float x,float y,float z);
void SetPositiong(laType* mat,float x,float y,float z);
void RotateByAxis(laType *mat,laType axis,float angle);

laType Addf(laType,float);
laType Subf(laType,float);
laType Mulf(laType,float);
laType Divf(laType,float);

laType Add(laType,laType);
laType Sub(laType,laType);
laType Mul(laType a,laType b);
laType Mulmp(laType *a,laType *b);
laType Mulmc(laType a,laType b);


void Transpose(laType*);
laType Transposed(laType);

laType Scalef(laType a,float b);
laType Vector2(float x,float y);
laType Vector(float x,float y,float z);
laType Vector4(float x,float y,float z,float w);
laType Matrix3x3(float,float,float,float,float,float,float,float,float);
laType Matrix4x4(laType,laType,laType,laType);
laType GetOrientation(laType mat);
laType GetPosition(laType mat);
laType GetPositionMatrix(laType mat);
laType GetOrientationTransposed(laType mat);
laType GetVectorTo(laType mat1,laType mat2);
laType GetNVectorTo(laType mat1,laType mat2);
void SetCameraDirection(laType *mat1,laType lookaxis);
laType LookAt(laType mat1,laType mat2,laAxis upaxis,laAxis lookaxis);
float Length(laType vector);

void LAPrint(laType);
float Determinant(laType mat);
float Minor(laType mat,uint8_t index);
laType Inverted(laType mat);
laType _Inverted(laType mat, char* filename, int line);
laType InvertedFast(laType mat);
laType Interpolate(laType mat1,laType mat2,float coeff);
laType InterpolateIn(laType mat1,laType mat2,float coeff);
laType ToEuler(laType mat);

void IdentityArray3x3(float* mat);
void InvertArray3x3(float* mat, float* result);
void MulArrays3x3(float* mat1, float* mat2, float* result);
void MulVectorByMatrixArray3x3(float* result, float* vector, float* matrix);
void MulMatrixByVectorArray3x3(float* result, float* matrix, float* vector);
void RotationArray3x3(float* result, float x);
float RotationFromArray3x3(float* matrix);

float laTypeGetItem(laType lat,int i);
void laTypeSetItem(laType *lat,int i,float val);
int laTypeGetType(laType lat);

#endif /* LINALG_H_ */
