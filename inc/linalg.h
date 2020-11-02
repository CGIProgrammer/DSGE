/*
 * linalg.h
 *
 *  Created on: 23 дек. 2017 г.
 *      Author: ivan
 */

#ifndef LINALG_H_
#define LINALG_H_

#ifdef __cplusplus
extern "C" {
#else
	#ifndef bool
		typedef _Bool bool;
	#endif
#endif

#include <math.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <float.h>

#define CHECK_TYPES
#define VECTOR4 4u
#define VECTOR 3u
#define VECTOR3 3u
#define VECTOR2 3u
#define COLOR 3u
#define UV 2u
#define MATRIX16 16u
#define MATRIX 16u
#define MATRIX9 9u

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

extern float laDet;

typedef enum
{
	laScalar = 0, laRowVector2, laRowVector3, laRowVector4,
	laColVector2, laMatrix2, laMatrix2x3, laMatrix2x4,
	laColVector3, laMatrix3x2, laMatrix3, laMatrix3x4,
	laColVector4, laMatrix4x2, laMatrix4x3, laMatrix4
} laType;

typedef struct
{
	double a[16];
	uint8_t type;
} laMatrixD;

typedef struct
{
	float a[16];
	uint8_t type;
} laMatrixS;

typedef laMatrixS laMatrix;

typedef enum
{
	laX = 0,
	laY,
	laZ,
	laXn,
	laYn,
	laZn
} laAxis;

extern const laAxis axes[6];

/*typedef struct
{
	float normal_coeff[4];
} laPlane;

float laPlaneGetDist(laPlane* plane, laType coords);
laPlane laPlaneMakeFromTriangle(laType a, laType b, laType c);*/

float fiSqrt(float x);

laMatrixS laTypeCastToSingle(laMatrix*);
laMatrixD laTypeCastToDouble(laMatrix*);

extern const laMatrix laIdentity;

laMatrix lsCross(laMatrix,laMatrix);
laMatrix laCrossn(laMatrix,laMatrix);
float laDot(laMatrix,laMatrix);
float laDotn(laMatrix a,laMatrix b);
void Normalize(laMatrix* mat);
laMatrix Normalized(laMatrix* mat);


/**
 * @brief Создание матрицы перспективной проекции.
 * Ширина и высота не обязательно в пикселях,
 * важно именно соотношение вертикали и горизонтали.
 * 
 * @param width Ширина
 * @param height Высота
 * @param zfar Расстояние дальнего отсечения (определяет дальность видимости)
 * @param znear Расстояние ближнего отсечения (больее ближние пиксели отсекаются)
 * @param angle Угол (в градусах) обзора по вертикали
 */
laMatrix laPerspective(float width,
					float height,
					float zfar,
					float znear,
					float angle);


/**
 * @brief Создание матрицы параллельной проекции.
 * Ширина и высота определяют только соотношение сторон.
 * 
 * @param width Ширина
 * @param height Высота
 * @param zfar Расстояние дальнего отсечения (определяет дальность видимости)
 * @param znear Расстояние ближнего отсечения (больее ближние пиксели отсекаются)
 * @param scale Поле зрения
 */
laMatrix laOrtho(
	float width,
	float height,
	float scale,
	float zfar,
	float znear
);

laMatrix laRotationX(float);
laMatrix laRotationY(float);
laMatrix laRotationZ(float);
laMatrix laRotationXYZ(float,float,float);
void laRotateXYZ(laMatrix*,float,float,float);
void laRotateXYZlocal(laMatrix*,float,float,float);
void laRotateXYZglobal(laMatrix*,float,float,float);
void laTranslatel(laMatrix* mat,float x,float y,float z);
void laTranslateg(laMatrix* mat,float x,float y,float z);
void laTranslate(laMatrix* mat,float x,float y,float z);
void laMatrixSetPosition(laMatrix* mat,float x,float y,float z);
void laMatrixSetPositionLocal(laMatrix* mat,float x,float y,float z);
void laRotateByAxis(laMatrix *mat,laMatrix axis,float angle);

laMatrix laAddf(laMatrix,float);
laMatrix laSubf(laMatrix,float);
laMatrix laMulf(laMatrix,float);
laMatrix laDivf(laMatrix,float);

laMatrix laAdd(laMatrix,laMatrix);
laMatrix laSub(laMatrix,laMatrix);
laMatrix laMul(laMatrix a,laMatrix b);
laMatrix Mulmp(laMatrix *a,laMatrix *b);
laMatrix Mulmc(laMatrix a,laMatrix b);

void laSetType(laMatrix* tensor, uint8_t type);

void Transpose(laMatrix*);
laMatrix Transposed(laMatrix);

laMatrix Scalef(laMatrix a,float b);
laMatrix Vector2(float x,float y);
laMatrix Vector(float x,float y,float z);
laMatrix Vector4(float x,float y,float z,float w);
laMatrix Matrix3x3(float,float,float,float,float,float,float,float,float);
laMatrix Matrix4x4(laMatrix,laMatrix,laMatrix,laMatrix);
laMatrix laMatrixGetOrientation(laMatrix mat);

laMatrix laMatrixGetPosition(laMatrix mat);
laMatrix laMatrixGetXDirection(laMatrix mat);
laMatrix laMatrixGetYDirection(laMatrix mat);
laMatrix laMatrixGetZDirection(laMatrix mat);

void laMatrixSetPosition(laMatrix* mat, float x, float y, float z);
void laMatrixSetXDirection(laMatrix* mat, float x, float y, float z);
void laMatrixSetYDirection(laMatrix* mat, float x, float y, float z);
void laMatrixSetZDirection(laMatrix* mat, float x, float y, float z);

laMatrix laMatrixGetPositionMatrix(laMatrix mat);
laMatrix laGetVectorTo(laMatrix mat1,laMatrix mat2);
laMatrix laGetNVectorTo(laMatrix mat1,laMatrix mat2);
void laMatrixSetDirection(laMatrix *mat1,laMatrix lookaxis);
laMatrix laLookAt(laMatrix mat1,laMatrix mat2,laAxis upaxis,laAxis lookaxis);
float Length(laMatrix vector);

void laPrint(laMatrix);
float laDeterminant(laMatrix mat);
float laMinor(laMatrix mat,uint8_t index);
laMatrix laInverted(laMatrix mat);
laMatrix laInterpolate(laMatrix mat1,laMatrix mat2,float coeff);
laMatrix laInterpolateIn(laMatrix mat1,laMatrix mat2,float coeff);
laMatrix ToEuler(laMatrix mat);

float laTypeGetItem(laMatrix lat,int i);
void laTypeSetItem(laMatrix *lat,int i,float val);
int laTypeGetType(laMatrix lat);
bool laCompare(laMatrix*, laMatrix*);
void laInvertArray(float *result, float *arr, int n);
void laWriteCSV(const char* name, float* mat, int n);
void laMulArrays(
		float* result,
		float* a, uint16_t ra, uint16_t ca,
		float* b, uint16_t rb, uint16_t cb);

#ifdef __cplusplus
}
#endif

#endif /* LINALG_H_ */
