/*
 * linalg.c
 *
 *  Created on: 23 дек. 2017 г.
 *      Author: ivan
 */

#include "linalg.h"
#include "memmanager.h"

#ifdef __cplusplus
extern "C" {
#else
typedef _Bool bool;
#endif

void (*segfault)(void) = 0;

const laMatrix laIdentity = {{1.0,0.0,0.0,0.0,
					  0.0,1.0,0.0,0.0,
					  0.0,0.0,1.0,0.0,
					  0.0,0.0,0.0,1.0},MATRIX};

static const uint8_t dimensions_a[16][2] = {
	{1, 1}, {1, 2}, {1, 3}, {1, 4},
	{2, 1}, {2, 2}, {2, 3}, {2, 4},
	{3, 1}, {3, 2}, {3, 3}, {3, 4},
	{4, 1}, {4, 2}, {4, 3}, {4, 4}
};

/*laPlane laPlaneMakeFromTriangle(laType a, laType b, laType c)
{
	laType normal = Crossn(Sub(c, a), Sub(b, a));
	float d = -normal.a[0] * a.a[0] - normal.a[1] * a.a[1] - normal.a[2] * a.a[2];
	laPlane result = {{normal.a[0], normal.a[1], normal.a[2], d}};
	return result;
}

float laPlaneGetDist(laPlane* plane, laType coords)
{
	return plane->normal_coeff[0] * coords.a[0] +
		   plane->normal_coeff[1] * coords.a[1] +
		   plane->normal_coeff[2] * coords.a[2] -
		   coords.a[3];
}*/

const laAxis axes[6] = {laX,laY,laZ,laXn,laYn,laZn};

float fiSqrt(float x) {
	float xhalf = 0.5f * x;
	void *integer = &x;
	uint32_t i = *(uint32_t*)integer;  // представим биты float в виде целого числа
	i = 0x5f3759df - (i >> 1);  // какого черта здесь происходит ?
	integer = &i;
	x = *(float*)integer;
	x = x*(1.5f-(xhalf*x*x));
	x = x*(1.5f-(xhalf*x*x));
	return x;
}

float fsqrt( float number )
{
	return 1.0/fiSqrt(number);
}

void laPrint(laMatrix la)
{
	if (la.type == MATRIX)
	{
		printf("Matrix /%f,\t%f,\t%f,\t%f\\\n"
			   "       |%f,\t%f,\t%f,\t%f|\n"
			   "       |%f,\t%f,\t%f,\t%f|\n"
			   "       \\%f,\t%f,\t%f,\t%f/\n",
			   la.a[0], la.a[1], la.a[2], la.a[3],
			   la.a[4], la.a[5], la.a[6], la.a[7],
			   la.a[8], la.a[9], la.a[10],la.a[11],
			   la.a[12],la.a[13],la.a[14],la.a[15]);
	}
	else if (la.type == VECTOR)
	{
		printf("LA 3D Vector (%.3f, %.3f, %.3f)\n",la.a[0],la.a[1],la.a[2]);
	}
	else if (la.type == VECTOR4)
	{
		printf("LA 4D Vector (%.3f, %.3f, %.3f, %.3f)\n",la.a[0],la.a[1],la.a[2],la.a[3]);
	}
	else
	{
		fprintf(stderr,"Wrong la type for print %d\n",la.type);
	}
}

laMatrixS laTypeCastToSingle(laMatrix* in)
{
	laMatrixS result = {{in->a[0], in->a[1], in->a[2], in->a[3],
					   in->a[4], in->a[5], in->a[6], in->a[7],
					   in->a[8], in->a[9], in->a[10],in->a[11],
					   in->a[12],in->a[13],in->a[14],in->a[15]},in->type};
	return result;
}

laMatrixD laTypeCastToDouble(laMatrix* in)
{
	laMatrixD result = {{in->a[0], in->a[1], in->a[2], in->a[3],
					   in->a[4], in->a[5], in->a[6], in->a[7],
					   in->a[8], in->a[9], in->a[10],in->a[11],
					   in->a[12],in->a[13],in->a[14],in->a[15]},in->type};
	return result;
}

laMatrix laAddf(laMatrix a,float b)
{
	laMatrix result;
	result.type = a.type;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		fprintf(stderr, "laType: invalid types for Addf\n");
		segfault();
	}
#endif

	for (unsigned i=0;i<a.type;i++)
	{
		result.a[i] = a.a[i] + b;
	}
	return result;
}

bool laCompare(laMatrix* a, laMatrix* b)
{
	if (a->type != b->type) return 0;
	for (int i=0; i<a->type; i++)
	{
		if (fabs(a->a[i]-b->a[i]) > FLT_EPSILON)
		{
			return 0;
		}
	}
	return 1;
}

laMatrix laSubf(laMatrix a,float b)
{
	laMatrix result;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		fprintf(stderr, "laType: invalid types for Subf\n");
		exit(-1);
	}
#endif

	result.type = a.type;
	for (unsigned i=0;i<a.type;i++)
	{
		result.a[i] = a.a[i] - b;
	}
	return result;
}

laMatrix laMulf(laMatrix a,float b)
{
	laMatrix result = a;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		fprintf(stderr, "laType: invalid type for Mulf (%d)\n", a.type);
		segfault();
	}
#endif

	result.type = a.type;
	for (unsigned i=0; (int)i<a.type - (a.type==VECTOR4); i++)
	{
		result.a[i] = a.a[i] * b;
	}
	return result;
}

laMatrix laDivf(laMatrix a,float b)
{
	laMatrix result;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		printf("Invalid types for Divf\n");
		segfault();
	}
#endif

	result.type = a.type;
	for (unsigned i=0; (int)i<a.type - (a.type==VECTOR4);i++)
	{
		result.a[i] = a.a[i] / b;
	}
	return result;
}
laMatrix laMul(laMatrix a,laMatrix b)
{
	laMatrix result = a;
	result.type = 0;
	uint16_t a_dims[2] = {dimensions_a[a.type-1][0], dimensions_a[a.type-1][1]};
	uint16_t b_dims[2] = {dimensions_a[b.type-1][0], dimensions_a[b.type-1][1]};
	if (b.type<a.type) {
		uint16_t w = b_dims[0];
		b_dims[0] = b_dims[1];
		b_dims[1] = w;
	}
	result.type = (b_dims[0]-1)*4 + a_dims[1];
	//printf("Mul %dx%d by %dx%d\n", a_dims[0], a_dims[1], b_dims[0], b_dims[1]);
	//printf("res %dx%d\n", dimensions_a[result.type-1][0], dimensions_a[result.type-1][1]);

	laMulArrays(result.a,
			a.a, a_dims[0], a_dims[1],
			b.a, b_dims[0], b_dims[1]);
	//LAPrint(result);
	return result;
}
/*laType laMul(laType a,laType b)
{
	laType result = a;
	a = b; b = result;
	result.type = 0;
	if (a.type==MATRIX && b.type==VECTOR)
	{
		result.type = VECTOR;
		result.a[0] = a.a[0]*b.a[0]+a.a[1]*b.a[1]+a.a[2]*b.a[2];
		result.a[1] = a.a[4]*b.a[0]+a.a[5]*b.a[1]+a.a[6]*b.a[2];
		result.a[2] = a.a[8]*b.a[0]+a.a[9]*b.a[1]+a.a[10]*b.a[2];
	}
	else if (a.type==VECTOR && b.type==MATRIX)
	{
		result.type = VECTOR;
		result.a[0] = b.a[0]*a.a[0]+b.a[1]*a.a[1]+b.a[2] *a.a[2];
		result.a[1] = b.a[4]*a.a[0]+b.a[5]*a.a[1]+b.a[6] *a.a[2];
		result.a[2] = b.a[8]*a.a[0]+b.a[9]*a.a[1]+b.a[10]*a.a[2];
	}
	else if (a.type==VECTOR4 && b.type==MATRIX)
	{
		result.type = VECTOR4;
		result.a[0] = b.a[0 ]*a.a[0]+b.a[1 ]*a.a[1]+b.a[2 ]*a.a[2]+b.a[3 ]*a.a[3];
		result.a[1] = b.a[4 ]*a.a[0]+b.a[5 ]*a.a[1]+b.a[6 ]*a.a[2]+b.a[7 ]*a.a[3];
		result.a[2] = b.a[8 ]*a.a[0]+b.a[9 ]*a.a[1]+b.a[10]*a.a[2]+b.a[11]*a.a[3];
		result.a[3] = b.a[12]*a.a[0]+b.a[13]*a.a[1]+b.a[14]*a.a[2]+b.a[15]*a.a[3];
	}
	else if (a.type==MATRIX && b.type==VECTOR4)
	{
		result.type = VECTOR4;
		result.a[0] = a.a[ 0]*b.a[0]+a.a[ 1]*b.a[1]+a.a[ 2]*b.a[2]+a.a[ 3]*b.a[3];
		result.a[1] = a.a[ 4]*b.a[0]+a.a[ 5]*b.a[1]+a.a[ 6]*b.a[2]+a.a[ 7]*b.a[3];
		result.a[2] = a.a[ 8]*b.a[0]+a.a[ 9]*b.a[1]+a.a[10]*b.a[2]+a.a[11]*b.a[3];
		result.a[3] = a.a[12]*b.a[0]+a.a[13]*b.a[1]+a.a[14]*b.a[2]+a.a[15]*b.a[3];
	}
	else if (a.type==MATRIX && b.type==MATRIX)
	{
		result.type = MATRIX;
		result.a[0] = a.a[0]*b.a[0] + a.a[1]*b.a[4] + a.a[2]*b.a[8] + a.a[3]*b.a[12];
		result.a[1] = a.a[0]*b.a[1] + a.a[1]*b.a[5] + a.a[2]*b.a[9] + a.a[3]*b.a[13];
		result.a[2] = a.a[0]*b.a[2] + a.a[1]*b.a[6] + a.a[2]*b.a[10]+ a.a[3]*b.a[14];
		result.a[3] = a.a[0]*b.a[3] + a.a[1]*b.a[7] + a.a[2]*b.a[11]+ a.a[3]*b.a[15];

		result.a[4] = a.a[4]*b.a[0] + a.a[5]*b.a[4] + a.a[6]*b.a[8] + a.a[7]*b.a[12];
		result.a[5] = a.a[4]*b.a[1] + a.a[5]*b.a[5] + a.a[6]*b.a[9] + a.a[7]*b.a[13];
		result.a[6] = a.a[4]*b.a[2] + a.a[5]*b.a[6] + a.a[6]*b.a[10]+ a.a[7]*b.a[14];
		result.a[7] = a.a[4]*b.a[3] + a.a[5]*b.a[7] + a.a[6]*b.a[11]+ a.a[7]*b.a[15];

		result.a[8] = a.a[8]*b.a[0] + a.a[9]*b.a[4] + a.a[10]*b.a[8] + a.a[11]*b.a[12];
		result.a[9] = a.a[8]*b.a[1] + a.a[9]*b.a[5] + a.a[10]*b.a[9] + a.a[11]*b.a[13];
		result.a[10]= a.a[8]*b.a[2] + a.a[9]*b.a[6] + a.a[10]*b.a[10]+ a.a[11]*b.a[14];
		result.a[11]= a.a[8]*b.a[3] + a.a[9]*b.a[7] + a.a[10]*b.a[11]+ a.a[11]*b.a[15];

		result.a[12]= a.a[12]*b.a[0]+ a.a[13]*b.a[4]+ a.a[14]*b.a[8] + a.a[15]*b.a[12];
		result.a[13]= a.a[12]*b.a[1]+ a.a[13]*b.a[5]+ a.a[14]*b.a[9] + a.a[15]*b.a[13];
		result.a[14]= a.a[12]*b.a[2]+ a.a[13]*b.a[6]+ a.a[14]*b.a[10]+ a.a[15]*b.a[14];
		result.a[15]= a.a[12]*b.a[3]+ a.a[13]*b.a[7]+ a.a[14]*b.a[11]+ a.a[15]*b.a[15];
	}
#ifdef CHECK_TYPES
	else
	{
		printf("Invalid types for multiplication %d %d\n",a.type,b.type);
		exit(-1);
	}
#endif
	return result;
}*/

laMatrix laAdd(laMatrix a,laMatrix b)
{
	laMatrix result;
	result.type = 0;
	if ((a.type==VECTOR4 || a.type==VECTOR3) && (a.type==VECTOR4 || b.type==VECTOR3))
	{
		a.a[0] += b.a[0];
		a.a[1] += b.a[1];
		a.a[2] += b.a[2];
		return a;
	}
	else if ((a.type==VECTOR3 || a.type==VECTOR4) && b.type==MATRIX16)
	{
		result = b;
		result.type = MATRIX16;
		result.a[3] = a.a[0] + b.a[3];
		result.a[7] = a.a[1] + b.a[7];
		result.a[11] = a.a[2] + b.a[11];
		return result;
	}
	else if (a.type==MATRIX16 && (b.type==VECTOR3 || b.type==VECTOR4))
	{
		result = a;
		result.type = MATRIX16;
		result.a[3]  = a.a[3]  + b.a[0];
		result.a[7]  = a.a[7]  + b.a[1];
		result.a[11] = a.a[11] + b.a[2];
		return result;
	}
	else if (a.type == b.type)
	{
		result.type = a.type;
		for (unsigned i=0;i<a.type;i++)
		{
			result.a[i] = a.a[i] + b.a[i];
		}
		return result;
	}
#ifdef CHECK_TYPES
	else
	{
		printf("Invalid types for summa\n");
		segfault();
	}
#endif
	return result;
}

laMatrix laSub(laMatrix a,laMatrix b)
{
	laMatrix result;
	result.type = 0;
	if (a.type==VECTOR4 && b.type==VECTOR4)
	{
		a.a[0] -= b.a[0];
		a.a[1] -= b.a[1];
		a.a[2] -= b.a[2];
		return a;
	}
	else if (a.type==VECTOR && b.type==MATRIX16)
	{
		result = b;
		result.type = MATRIX16;
		result.a[3] = a.a[0] - b.a[3];
		result.a[7] = a.a[1] - b.a[7];
		result.a[11] = a.a[2] - b.a[11];
		return result;
	}
	else if (a.type==MATRIX16 && b.type==VECTOR)
	{
		result = a;
		result.type = MATRIX16;
		result.a[3]  = a.a[3]  - b.a[0];
		result.a[7]  = a.a[7]  - b.a[1];
		result.a[11] = a.a[11] - b.a[2];
		return result;
	}
	else if (a.type == b.type)
	{
		result.type = a.type;
		for (unsigned i=0;i<a.type;i++)
		{
			result.a[i] = a.a[i] - b.a[i];
		}
		return result;
	}
#ifdef CHECK_TYPES
	else
	{
		printf("Invalid types for summa\n");
		segfault();
	}
#endif
	return result;
}

laMatrix lsCross(laMatrix a,laMatrix b)
{
#ifdef CHECK_TYPES
	if ((a.type!=VECTOR && a.type!=VECTOR4) || (b.type!=VECTOR && b.type!=VECTOR4))
	{
		printf("Invalid types for cross\n");
		segfault();
	}
#endif
	laMatrix result;
	result.type = 0;
	result.type = VECTOR;
	result.a[0] = a.a[1]*b.a[2] - b.a[1]*a.a[2];
	result.a[1] = -(a.a[0]*b.a[2] - b.a[0]*a.a[2]);
	result.a[2] = a.a[0]*b.a[1] - b.a[0]*a.a[1];
	return result;
}

laMatrix laCrossn(laMatrix a,laMatrix b)
{
	laMatrix result = lsCross(a,b);
	float length = fiSqrt(result.a[0]*result.a[0]+result.a[1]*result.a[1]+result.a[2]*result.a[2]);
	result.a[0] *= length;
	result.a[1] *= length;
	result.a[2] *= length;
	return result;
}

float laDot(laMatrix a,laMatrix b)
{
	if ((a.type == VECTOR || a.type == VECTOR4) && (b.type == VECTOR || b.type == VECTOR4))
	{
		return a.a[0]*b.a[0] + a.a[1]*b.a[1] + a.a[2]*b.a[2];
	}

#ifdef CHECK_TYPES
	printf("Invalid types for dot\n");
	exit(-1);
#endif
}

float laDotn(laMatrix a,laMatrix b)
{
	return laDot(a,b)*fiSqrt(a.a[0]*a.a[0]+a.a[1]*a.a[1]+a.a[2]*a.a[2])
				   *fiSqrt(b.a[0]*b.a[0]+b.a[1]*b.a[1]+b.a[2]*b.a[2]);
}

laMatrix laPerspective(float width,
					float height,
					float zfar,
					float znear,
					float angle)
{
	angle = 1.0f/tanf(radians(angle/2.0));
	laMatrix result = {{angle,0.0f,0.0f,0.0f,
					  0.0f,angle/height*width,0.0f,0.0f,
					  0.0f,0.0f,-(znear+zfar)/(zfar-znear),-2.0f*znear*zfar/(zfar-znear),
					  0.0f,0.0f,-1.0f,0.0f},MATRIX};
	return result;
}

laMatrix laOrtho(
	float width,
	float height,
	float scale,
	float zfar,
	float znear
)
{
	float dz = zfar-znear;
	scale = 2.0f / scale;
	laMatrix result = {{scale/width*height,	0.0f,	0.0f,		0.0f,
					  0.0f,		scale/height*width,	0.0f,		0.0f,
					  0.0f,		0.0f,		-2.0f/(dz),	(-dz)/(dz),
					  0.0f,		0.0f,		0.0f,		1.0f},MATRIX};
	return result;
}

laMatrix laRotationX(float angle)
{
	laMatrix result;
	result.type = MATRIX;
	result.a[0] = 1;
	result.a[1] = 0;
	result.a[2] = 0;
	result.a[3] = 0;

	result.a[4] = 0;
	result.a[5] = cosf(angle);
	result.a[6] =-sinf(angle);
	result.a[7] = 0;

	result.a[8] = 0;
	result.a[9] = sinf(angle);
	result.a[10]= cosf(angle);
	result.a[11]= 0;

	result.a[12]= 0;
	result.a[13]= 0;
	result.a[14]= 0;
	result.a[15]= 1;
	return result;
}

laMatrix laRotationY(float angle)
{
	laMatrix result;
	result.type = MATRIX;
	result.a[0] = cosf(angle);
	result.a[1] = 0.0;
	result.a[2] = sinf(angle);
	result.a[3] = 0.0;

	result.a[4] = 0.0;
	result.a[5] = 1.0;
	result.a[6] = 0.0;
	result.a[7] = 0.0;

	result.a[8] =-sinf(angle);
	result.a[9] = 0.0;
	result.a[10]= cosf(angle);
	result.a[11]= 0.0;

	result.a[12]= 0.0;
	result.a[13]= 0.0;
	result.a[14]= 0.0;
	result.a[15]= 1.0;
	return result;
}

laMatrix laRotationZ(float angle)
{
	laMatrix result;
	result.type = MATRIX;
	result.a[0] = cosf(angle);
	result.a[1] =-sinf(angle);
	result.a[2] = 0.0;
	result.a[3] = 0.0;

	result.a[4] = sinf(angle);
	result.a[5] = cosf(angle);
	result.a[6] = 0.0;
	result.a[7] = 0.0;

	result.a[8] = 0.0;
	result.a[9] = 0.0;
	result.a[10]= 1.0;
	result.a[11]= 0.0;

	result.a[12]= 0.0;
	result.a[13]= 0.0;
	result.a[14]= 0.0;
	result.a[15]= 1.0;
	return result;
}

laMatrix laRotationXYZ(float x,float y,float z)
{
	return laMul(laRotationZ(z),laMul(laRotationY(y),laRotationX(x)));
}

void laRotateXYZ(laMatrix* mat,float x,float y,float z)
{
	*mat = laMul(*mat, laRotationXYZ(x,y,z));
}

void laRotateXYZlocal(laMatrix* mat,float x,float y,float z)
{
	float px = mat->a[3],py = mat->a[7],pz = mat->a[11];

#ifdef CHECK_TYPES
	if (mat->type!=3 && mat->type!=4 && mat->type!=9 && mat->type!=16)
	{
		fprintf(stderr, "laType: invalid types for Divf\n");
		exit(-1);
	}
#endif

	mat->a[3] = 0;
	mat->a[7] = 0;
	mat->a[11]= 0;
	*mat = laMul(*mat, laRotationXYZ(x,y,z));
	mat->a[3] = px;
	mat->a[7] = py;
	mat->a[11]= pz;
}

void laRotateXYZglobal(laMatrix* mat,float x,float y,float z)
{
	float px = mat->a[3],py = mat->a[7],pz = mat->a[11];
	mat->a[3] = 0;
	mat->a[7] = 0;
	mat->a[11]= 0;
	*mat = laMul(laRotationXYZ(x,y,z), *mat);
	mat->a[3] = px;
	mat->a[7] = py;
	mat->a[11]= pz;
}

void laRotateByAxis(laMatrix *mat,laMatrix axis,float angle)
{
	float x=axis.a[0], y=axis.a[1], z=axis.a[2];
	axis.a[0] = mat->a[3];
	axis.a[1] = mat->a[7];
	axis.a[2] = mat->a[11];
	mat->a[3] = mat->a[7] = mat->a[11] = 0.0;
	float length = fiSqrt(x*x+y*y+z*z);
	x *= length;
	y *= length;
	z *= length;
	float ca=cosf(angle),sa=sinf(angle);
	*mat = laMul(Matrix3x3(ca+(1.0-ca)*x*x,  (1.0-ca)*x*y-sa*z,  (1-ca)*x*z+sa*y,
						(1.0-ca)*y*x+sa*z, ca+(1.0-ca)*y*y,    (1.0-ca)*y*z-sa*x,
						(1.0-ca)*z*x-sa*y, (1.0-ca)*z*y+sa*x,  ca+(1.0-ca)*z*z), *mat);
	mat->a[3] = axis.a[0];
	mat->a[7] = axis.a[1];
	mat->a[11]= axis.a[2];
}

void laTranslatel(laMatrix* mat,float x,float y,float z)
{
	*mat = laMul(*mat,laAdd(laIdentity,Vector(x,y,z)));
}

void laTranslate(laMatrix* mat,float x,float y,float z)
{
	*mat = laMul(*mat,laAdd(laIdentity,Vector(x,y,z)));
}

void laTranslateg(laMatrix* mat,float x,float y,float z)
{
	*mat = laMul(laAdd(laIdentity,Vector(x,y,z)), *mat);
}

void laMatrixSetPositionLocal(laMatrix* mat,float x,float y,float z)
{
	mat->a[3] = 0;
	mat->a[7] = 0;
	mat->a[11] = 0;
	laTranslateg(mat,x,y,z);
}

laMatrix Vector2(float x,float y)
{
	laMatrix result;
	memset(&result,0,sizeof(result));
	result.type = VECTOR2;
	result.a[0] = x;
	result.a[1] = y;
	return result;
}

laMatrix Vector(float x,float y,float z)
{
	laMatrix result;
	result.type = VECTOR4;
	result.a[0] = x;
	result.a[1] = y;
	result.a[2] = z;
	result.a[3] = 0;
	return result;
}
laMatrix Vector4(float x,float y,float z,float w)
{
	laMatrix result;
	result.type = VECTOR4;
	result.a[0] = x;
	result.a[1] = y;
	result.a[2] = z;
	result.a[3] = w;
	return result;
}

laMatrix Matrix3x3(float a,float b,float c,
				  float d,float e,float f,
				  float g,float h,float i)
{
	laMatrix result = laIdentity;
	result.a[0] = a;
	result.a[1] = b;
	result.a[2] = c;
	result.a[4] = d;
	result.a[5] = e;
	result.a[6] = f;
	result.a[8] = g;
	result.a[9] = h;
	result.a[10]= i;
	result.a[15] = 1.0;
	return result;
}

laMatrix Matrix4x4(laMatrix vector1,
				 laMatrix vector2,
				 laMatrix vector3,
				 laMatrix vector4)
{
	if (vector1.type!=4) fprintf(stderr,"Warning: 1 argument has wrong type\n");
	if (vector2.type!=4) fprintf(stderr,"Warning: 2 argument has wrong type\n");
	if (vector3.type!=4) fprintf(stderr,"Warning: 3 argument has wrong type\n");
	if (vector4.type!=4) fprintf(stderr,"Warning: 4 argument has wrong type\n");
	laMatrix result = {
	{
			vector1.a[0],vector2.a[0],vector3.a[0],vector4.a[0],
			vector1.a[1],vector2.a[1],vector3.a[1],vector4.a[1],
			vector1.a[2],vector2.a[2],vector3.a[2],vector4.a[2],
			vector1.a[3],vector2.a[3],vector3.a[3],vector4.a[3]
	}, MATRIX};
	return result;
}

laMatrix _iMatrix3x3(float a,float b,float c,
				  float d,float e,float f,
				  float g,float h,float i)
{
	laMatrix result = laIdentity;
	result.a[0] = a;
	result.a[1] = b;
	result.a[2] = c;
	result.a[4] = d;
	result.a[5] = e;
	result.a[6] = f;
	result.a[8] = g;
	result.a[9] = h;
	result.a[10]= i;
	result.a[15] = 1.0;
	return result;
}

laMatrix Scalef(laMatrix a,float b)
{
	a.a[0] *= b;
	a.a[1] *= b;
	a.a[2] *= b;

	a.a[4] *= b;
	a.a[5] *= b;
	a.a[6] *= b;

	a.a[8] *= b;
	a.a[9] *= b;
	a.a[10] *= b;
	return a;
}

inline static void swap(float* a,float* b)
{
	float w = *a;
	*a = *b;
	*b = w;
}

laMatrix Normalized(laMatrix* mat)
{
	laMatrix result = *mat;
	Normalize(&result);
	return result;
}

void Normalize(laMatrix* vector)
{
#ifdef CHECK_TYPES
	if (vector->type!=VECTOR && vector->type!=VECTOR4)
	{
		printf("Invalid type for Normalize (%d)\n",vector->type);
		exit(-1);
	}
#endif
	float length = fiSqrt(vector->a[0]*vector->a[0]+vector->a[1]*vector->a[1]+vector->a[2]*vector->a[2]);
	if (length!=INFINITY)
	{
		vector->a[0] *= length;
		vector->a[1] *= length;
		vector->a[2] *= length;
	}
}


void Transpose(laMatrix* mat)
{
	if (mat->type!=MATRIX)
	{
		printf("Invalid type (%d) for transpose\n",mat->type);
		exit(-1);
	}
	swap(&mat->a[1],&mat->a[4]);
	swap(&mat->a[2],&mat->a[8]);
	swap(&mat->a[3],&mat->a[12]);
	swap(&mat->a[6],&mat->a[9]);
	swap(&mat->a[7],&mat->a[13]);
	swap(&mat->a[11],&mat->a[14]);
}

laMatrix Transposed(laMatrix mat)
{
	Transpose(&mat);
	return mat;
}

float laMinor(laMatrix mat,uint8_t index)
{
	index &= 0xF;
	uint8_t Aj = (index>>2)&3;
	uint8_t Ai = index& 3;
	if (12-Aj*4)
	{
		memmove(&mat.a[4*Aj],&mat.a[4*(Aj+1)],(12-Aj*4)*4);
	}

	if (3-Ai)
	{
		memmove(&mat.a[Ai],   &mat.a[Ai   +1],(3-Ai)*4);
		memmove(&mat.a[Ai+4], &mat.a[Ai+4 +1],(3-Ai)*4);
		memmove(&mat.a[Ai+8], &mat.a[Ai+8 +1],(3-Ai)*4);
	}

	return
			mat.a[A11]*mat.a[A22]*mat.a[A33]+
			mat.a[A12]*mat.a[A23]*mat.a[A31]+
			mat.a[A13]*mat.a[A32]*mat.a[A21]-
			mat.a[A31]*mat.a[A22]*mat.a[A13]-
			mat.a[A12]*mat.a[A21]*mat.a[A33]-
			mat.a[A23]*mat.a[A32]*mat.a[A11];
}

inline float laDeterminant(laMatrix mat)
{
	if (mat.type!=MATRIX)
	{
		printf("Invalid type (%d) for determinant\n",mat.type);
		exit(-1);
	}
	return mat.a[A11]*laMinor(mat,A11)-mat.a[A12]*laMinor(mat,A12)+mat.a[A13]*laMinor(mat,A13)-mat.a[A14]*laMinor(mat,A14);
}

/*laType Inverted(laType mat)
{
	float det = Determinant(mat);
	laType connected = laIdentity;
	for (uint8_t i=0;i<16;i++)
	{
		connected.a[i] = Minor(mat,i)*(((~i ^ ~(i>>2))&1) ? -1.0 : 1.0) / det;
	}
	Transpose(&connected);

	return connected;
}*/
float laDet = 0.0f;

laMatrix laInverted(laMatrix mat)
{
	if (mat.type != 16) {
		fprintf(stderr, "Error: invalid type for Inverted()\n");
		exit(-1);
	}
	float(*m)[4];
	m = (float(*)[4])mat.a;
	//memcpy(m, mat.a, sizeof(m));
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

		if (det==0.0)
		{
			fprintf(stderr, "Warning: laType Inverted() have zero determinant\n");
		}
		laDet = det;

		mat.a[0] = (a11 * b11 - a12 * b10 + a13 * b09) / det;
		mat.a[1] = (a02 * b10 - a01 * b11 - a03 * b09) / det;
		mat.a[2] = (a31 * b05 - a32 * b04 + a33 * b03) / det;
		mat.a[3] = (a22 * b04 - a21 * b05 - a23 * b03) / det;
		mat.a[4] = (a12 * b08 - a10 * b11 - a13 * b07) / det;
		mat.a[5] = (a00 * b11 - a02 * b08 + a03 * b07) / det;
		mat.a[6] = (a32 * b02 - a30 * b05 - a33 * b01) / det;
		mat.a[7] = (a20 * b05 - a22 * b02 + a23 * b01) / det;
		mat.a[8] = (a10 * b10 - a11 * b08 + a13 * b06) / det;
		mat.a[9] = (a01 * b08 - a00 * b10 - a03 * b06) / det;
		mat.a[10] = (a30 * b04 - a31 * b02 + a33 * b00) / det;
		mat.a[11] = (a21 * b02 - a20 * b04 - a23 * b00) / det;
		mat.a[12] = (a11 * b07 - a10 * b09 - a12 * b06) / det;
		mat.a[13] = (a00 * b09 - a01 * b07 + a02 * b06) / det;
		mat.a[14] = (a31 * b01 - a30 * b03 - a32 * b00) / det;
		mat.a[15] = (a20 * b03 - a21 * b01 + a22 * b00) / det;
		return mat;
}

laMatrix laMatrixGetOrientation(laMatrix mat)
{
	return Matrix3x3(mat.a[0],mat.a[1],mat.a[2],mat.a[4],mat.a[5],mat.a[6],mat.a[8],mat.a[9],mat.a[10]);
}

laMatrix laMatrixGetPosition(laMatrix mat)
{
	//mat = Mul(GetOrientationTransposed(mat),mat);
	laMatrix vector = {{mat.a[3],mat.a[7],mat.a[11],1.0},VECTOR4};
	return vector;
}

laMatrix laMatrixGetXDirection(laMatrix mat)
{
	return Vector(mat.a[0],mat.a[4],mat.a[8]);
}

laMatrix laMatrixGetYDirection(laMatrix mat)
{
	return Vector(mat.a[1],mat.a[5],mat.a[9]);
}

laMatrix laMatrixGetZDirection(laMatrix mat)
{
	return Vector(mat.a[2],mat.a[6],mat.a[10]);
}

void laMatrixSetPosition(laMatrix* mat, float x, float y, float z)
{
	mat->a[3]  = x;
	mat->a[7]  = y;
	mat->a[11] = z;
}
void laMatrixSetXDirection(laMatrix* mat, float x, float y, float z)
{
	mat->a[0] = x;
	mat->a[4] = y;
	mat->a[8] = z;
}
void laMatrixSetYDirection(laMatrix* mat, float x, float y, float z)
{
	mat->a[1] = x;
	mat->a[5] = y;
	mat->a[9] = z;
}
void laMatrixSetZDirection(laMatrix* mat, float x, float y, float z)
{
	mat->a[2]  = x;
	mat->a[6]  = y;
	mat->a[10] = z;
}

laMatrix laMatrixGetPositionMatrix(laMatrix mat)
{
	mat.a[9] = mat.a[8] = mat.a[6] = mat.a[5] = mat.a[2] = mat.a[1] = 0.0;
	mat.a[10] = mat.a[5] = mat.a[0] = 1.0;
	return mat;
	//return Mul(GetOrientationTransposed(mat),mat);
}

laMatrix laGetVectorTo(laMatrix mat1,laMatrix mat2)
{
	//return GetPosition(Mul(Inverted(mat1),mat2));
	return laSub(laMatrixGetPosition(mat2),laMatrixGetPosition(mat1));
}

laMatrix laGetNVectorTo(laMatrix mat1,laMatrix mat2)
{
	mat1 = laSub(laMatrixGetPosition(mat2),laMatrixGetPosition(mat1));
	float length = fiSqrt(mat1.a[0]*mat1.a[0]+mat1.a[1]*mat1.a[1]+mat1.a[2]*mat1.a[2]);
	mat1.a[0] *= length;
	mat1.a[1] *= length;
	mat1.a[2] *= length;
	mat1.type = VECTOR;
	return mat1;
}

float Length(laMatrix vector)
{
	if (vector.type!=VECTOR && vector.type!=VECTOR4)
	{
		printf("Invalid type (%d) for length\n",vector.type);
		exit(-1);
	}
	return fsqrt(vector.a[0]*vector.a[0]+vector.a[1]*vector.a[1]+vector.a[2]*vector.a[2]);
}

void laMatrixSetDirection(laMatrix *mat1,laMatrix lookaxis)
{
	if (lookaxis.type!=VECTOR)
	{
		fprintf(stderr,"SetMatrixDirection argument lookaxis should be a VECTOR\n");
		exit(-1);
	}
	if (mat1->type!=MATRIX)
	{
		fprintf(stderr,"SetMatrixDirection argument mat1 should be a MATRIX\n");
		exit(-1);
	}
	
	laMatrix rightDirection = laCrossn(lookaxis,Vector(0.0,0.0,-1.0));
	laMatrix upDirection = laCrossn(lookaxis,rightDirection);
	mat1->a[0] = rightDirection.a[0];
	mat1->a[4] = rightDirection.a[1];
	mat1->a[8] = rightDirection.a[2];
	mat1->a[1] = upDirection.a[0];
	mat1->a[5] = upDirection.a[1];
	mat1->a[9] = upDirection.a[2];
	mat1->a[2] = lookaxis.a[0];
	mat1->a[6] = lookaxis.a[1];
	mat1->a[10]= lookaxis.a[2];
}

laMatrix laLookAt(laMatrix mat1,laMatrix mat2,laAxis upaxis,laAxis lookaxis)
{
	laMatrix look_axes[] = {
			{{ 1.0, 0.0, 0.0},VECTOR}, {{0.0, 1.0, 0.0},VECTOR}, {{0.0, 0.0, 1.0},VECTOR},
			{{-1.0, 0.0, 0.0},VECTOR}, {{0.0,-1.0, 0.0},VECTOR}, {{0.0, 0.0,-1.0},VECTOR}
	};

	laMatrix Eye = laMatrixGetPosition(mat1);
	laMatrix At  = laMatrixGetPosition(mat2);
	laMatrix zaxis = laSub(Eye, At); Normalize(&zaxis);
	laMatrix Up = look_axes[upaxis];//	Up.a[upaxis%3] = 1.0 - 2.0*(upaxis>laZ);
	laMatrix xaxis = laCrossn(Up, zaxis);
	laMatrix yaxis = laCrossn(zaxis, xaxis);

	switch (lookaxis)
	{
		case laZ : {
			laMatrix result = {{
			 xaxis.a[0],    yaxis.a[0],    zaxis.a[0],   mat1.a[3],
			 xaxis.a[1],    yaxis.a[1],    zaxis.a[1],   mat1.a[7],
			 xaxis.a[2],    yaxis.a[2],    zaxis.a[2],   mat1.a[11],
			 0.0,           0.0,        0.0,             1.0
			}, 16};
			return result;
		}
		case laX : {
			laMatrix result = {{
			 zaxis.a[0],    xaxis.a[0],    yaxis.a[0],   mat1.a[3],
			 zaxis.a[1],    xaxis.a[1],    yaxis.a[1],   mat1.a[7],
			 zaxis.a[2],    xaxis.a[2],    yaxis.a[2],   mat1.a[11],
			 0.0,           0.0,        0.0,             1.0
			}, 16};
			return result;
		}
		case laY : {
			laMatrix result = {{
			 yaxis.a[0],    zaxis.a[0],    xaxis.a[0],   mat1.a[3],
			 yaxis.a[1],    zaxis.a[1],    xaxis.a[1],   mat1.a[7],
			 yaxis.a[2],    zaxis.a[2],    xaxis.a[2],   mat1.a[11],
			 0.0,           0.0,        0.0,             1.0
			}, 16};
			return result;
		}
		default : return laIdentity;
	}
	return laIdentity;
	/*laType vector_to = GetNVectorTo(mat1,mat2);
	laType axis = {{0,0,0},VECTOR};	axis.a[upaxis%3] = 1.0 - 2.0*(upaxis>laZ);
	laType third;
	third = Crossn(vector_to,axis);
	axis = Cross(third,vector_to);
	lookaxis %= 6;

	if (lookaxis==laX)
	{
		laType result = {{ vector_to.a[0],axis.a[0],third.a[0],mat1.a[3],
						   vector_to.a[1],axis.a[1],third.a[1],mat1.a[7],
						   vector_to.a[2],axis.a[2],third.a[2],mat1.a[11],
						   0.0			 ,0.0		,0.0	  ,1.0		},MATRIX};
		return result;
	}
	if (lookaxis==laY)
	{
		laType result = {{ third.a[0],vector_to.a[0],axis.a[0],mat1.a[3],
						   third.a[1],vector_to.a[1],axis.a[1],mat1.a[7],
						   third.a[2],vector_to.a[2],axis.a[2],mat1.a[11],
						   0.0			 ,0.0		,0.0	  ,1.0		},MATRIX};
		return result;
	}
	if (lookaxis==laZ)
	{
		laType result = {{ axis.a[0],third.a[0],vector_to.a[0],mat1.a[3],
						   axis.a[1],third.a[1],vector_to.a[1],mat1.a[7],
						   axis.a[2],third.a[2],vector_to.a[2],mat1.a[11],
						   0.0			 ,0.0		,0.0	  ,1.0		},MATRIX};
		return result;
	}

	if (lookaxis==laXn)
	{
		laType result = {{-vector_to.a[0],axis.a[0],-third.a[0],mat1.a[3],
						  -vector_to.a[1],axis.a[1],-third.a[1],mat1.a[7],
						  -vector_to.a[2],axis.a[2],-third.a[2],mat1.a[11],
						   0.0			 ,0.0		,0.0	  ,1.0		},MATRIX};
		return result;
	}
	if (lookaxis==laYn)
	{
		laType result = {{-third.a[0],-vector_to.a[0],axis.a[0],mat1.a[3],
						  -third.a[1],-vector_to.a[1],axis.a[1],mat1.a[7],
						  -third.a[2],-vector_to.a[2],axis.a[2],mat1.a[11],
						   0.0			 ,0.0		,0.0	  ,1.0		},MATRIX};
		return result;
	}
	if (lookaxis==laZn)
	{
		laType result = {{ axis.a[0],-third.a[0],-vector_to.a[0],mat1.a[3],
						   axis.a[1],-third.a[1],-vector_to.a[1],mat1.a[7],
						   axis.a[2],-third.a[2],-vector_to.a[2],mat1.a[11],
						   0.0			 ,0.0		,0.0	  ,1.0		},MATRIX};
		return result;
	}
	return mat1;*/
}

laMatrix laInterpolate(laMatrix mat1,laMatrix mat2,float coeff)
{
    if (coeff>1.0) coeff = 1.0;
    if (coeff<0.0) coeff = 0.0;
    laMatrix result;
    result = laAdd(laMulf(mat1,1.0-coeff),laMulf(mat2,coeff));

    laMatrix v1 = Vector(result.a[0], result.a[4], result.a[ 8]);
    laMatrix v2 = Vector(result.a[1], result.a[5], result.a[ 9]);
    laMatrix v3 = Vector(result.a[2], result.a[6], result.a[10]);

    v3 = laCrossn(v1,v2);
    v2 = laCrossn(v3,v1);
    v1 = laCrossn(v2,v3);

    result.a[0] = v1.a[0];
    result.a[4] = v1.a[1];
    result.a[8] = v1.a[2];

    result.a[1] = v2.a[0];
    result.a[5] = v2.a[1];
    result.a[9] = v2.a[2];

    result.a[2] = v3.a[0];
    result.a[6] = v3.a[1];
    result.a[10]= v3.a[2];

    return result;
}

laMatrix laInterpolateIn(laMatrix mat1,laMatrix mat2,float coeff)
{
	/*laType result = Mulmc( mat1, Interpolate(Identity, Mulmc(mat2,Inverted(mat1)), coeff) );
	result.a[3] = mat1.a[3]*(1.0-coeff) + mat2.a[3]*coeff;
	result.a[7] = mat1.a[7]*(1.0-coeff) + mat2.a[7]*coeff;
	result.a[11] = mat1.a[11]*(1.0-coeff) + mat2.a[11]*coeff;*/
	return laAdd(laMulf(mat1,1.0-coeff),laMulf(mat2,coeff));
}
/*
11 12 13 14
21 22 23 24
31 32 33 34
41 42 43 44
*/

laMatrix ToEuler(laMatrix in_mat)
{
	laMatrix result = {{0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0},VECTOR};
	float (*mat)[4] = (float(*)[4])in_mat.a;
	float* eul = result.a;
	float eul1[3];
	float eul2[3];
	float cy = sqrtf(mat[0][0]*mat[0][0] + mat[0][1]*mat[0][1]);

	if (cy > 16.0f * FLT_EPSILON) {

		eul1[0] = -atan2f(mat[1][2], mat[2][2]);
		eul1[1] = -atan2f(-mat[0][2], cy);
		eul1[2] = -atan2f(mat[0][1], mat[0][0]);

		eul2[0] = -atan2f(-mat[1][2], -mat[2][2]);
		eul2[1] = -atan2f(-mat[0][2], -cy);
		eul2[2] = -atan2f(-mat[0][1], -mat[0][0]);

	}
	else {
		eul1[0] = atan2f(-mat[2][1], mat[1][1]);
		eul1[1] = atan2f(-mat[0][2], cy);
		eul1[2] = 0.0f;

		memcpy(eul2, eul1, sizeof(float[3]));
	}

	if (fabsf(eul1[0]) + fabsf(eul1[1]) + fabsf(eul1[2]) > fabsf(eul2[0]) + fabsf(eul2[1]) + fabsf(eul2[2])) {
		memcpy(eul, eul2, sizeof(float[3]));
	}
	else {
		memcpy(eul, eul1, sizeof(float[3]));
	}

	return result;
}
/*
laType ToEuler(laType matrix)
{
	laType result = {{0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0},0};
	float *res = result.a;
	float (*mat)[4] = (float(*)[4])matrix.a;
	if (matrix.type==MATRIX)
	{
		result.type = VECTOR;
		if (mat[1][0] > 0.998) { // singularity at north pole
			res[1] = atan2f(mat[0][2],mat[2][2]);
			res[0] = 3.1415926535/2;
			res[2] = 0;
		}
		else if (mat[1][0] < -0.998) { // singularity at south pole
			res[1] = atan2f(mat[0][2],mat[2][2]);
			res[0] = -3.1415926535/2;
			res[2] = 0;
		}
		else
		{
			res[1] = atan2f(-mat[2][0],mat[0][0]);
			res[0] = atan2f(-mat[1][2],mat[1][1]);
			res[2] = asinf(mat[1][0]);
		}
	}

	return result;
}*/

float laTypeGetItem(laMatrix lat,int i)
{
	return lat.a[i];
}
void laTypeSetItem(laMatrix *lat,int i,float val)
{
	lat->a[i] = val;
}
int laTypeGetType(laMatrix lat)
{
	return lat.type;
}

void laSetType(laMatrix* tensor, uint8_t type)
{
	tensor->type = type;
}

void laWriteCSV(const char* name, float* mat, int n)
{
	float (*a)[n] = (float(*)[n])mat;
	FILE* fp = fopen(name, "w");
	for (int i=0; i<n; i++) {
		for (int j=0; j<n; j++) {
			fprintf(fp, "%.4f%c", a[i][j], ((j+1)==n) ? '\n' : '\t');
		}
	}
	fclose(fp);
}

void laMulArrays(
		float* result,
		float* a, uint16_t ra, uint16_t ca,
		float* b, uint16_t rb, uint16_t cb)
{
	if (ca!=rb) {
		fprintf(stderr, "Invalid types for Mul (%dx%d <-> %dx%d)\n",
				(int)ra, (int)ca,
				(int)rb, (int)cb);
		segfault();
	}
	float (*a_mat)[ca] = (float(*)[ca])a;
	float (*b_mat)[cb] = (float(*)[cb])b;
	float (*r_mat)[ca] = (float(*)[ca])result;
	for (uint16_t i=0; i<ca; i++) {
		for (uint16_t j=0; j<ra; j++) {
			r_mat[i][j] = 0;
			for (uint16_t k=0; k<rb; k++) {
				r_mat[i][j] += a_mat[i][k] * b_mat[k][j];
			}
		}
	}
}

void laInvertArray(float *result, float *arr, int n)
{
	double (*a)[n] = (double(*)[n])sMalloc(sizeof(double[n][n]));
	double (*b)[n] = (double(*)[n])sMalloc(sizeof(double[n][n]));
	float (*orig)[n] = (float(*)[n])arr;
	double max;

	for (int i=0; i<n; i++) {
		for (int j=0; j<n; j++) {
			a[i][j] = orig[i][j];
			b[i][j] = i==j;
		}
	}
	for (int k=0; k<n; k++) {
		int index;
		// Поиск столбца с максимальным a[k][i]
		max = a[k][k];
		index = k;
		if (max==0) {
			for (int i = k + 1; i < n; i++) {
				if (fabs(a[k][i]) > fabs(max)) {
					max = a[k][i];
					index = i;
				}
			}
		}

		if (max == 0) {
			printf("Решение получить невозможно из-за нулевой строки %d матрицы A\n", index);
			laWriteCSV("fail.csv", (float*)a, n);
			exit(0);
		}
		
		// Перестановка столбцов k и index
		if (index!=k) {
			for (int j = 0; j < n; j++) {
				double temp = a[j][index];
				a[j][index] = a[j][k];
				a[j][k] = temp;

				temp = b[j][index];
				b[j][index] = b[j][k];
				b[j][k] = temp;
			}
		}
		for (int j = 0; j < n; j++) {
			a[k][j] /= max;
			b[k][j] /= max;
		}
		for (int i = 0; i < n; i++) {
			double temp = a[i][k];
			if (i==k) continue;
			for (int j = 0; j < n; j++) {
				a[i][j] -= a[k][j] * temp;
				b[i][j] -= b[k][j] * temp;
			}
		}
	}
	for (int i=0; i<n*n; i++) {
		result[i] = ((double*)b)[i];
	}
	sDelete(a);
	sDelete(b);
}


#ifdef __cplusplus
}
#endif