/*
 * linalg.c
 *
 *  Created on: 23 дек. 2017 г.
 *      Author: ivan
 */

#include "linalg.h"

const laType Identity = {{1.0,0.0,0.0,0.0,
					  0.0,1.0,0.0,0.0,
					  0.0,0.0,1.0,0.0,
					  0.0,0.0,0.0,1.0},MATRIX};

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

void LAPrint(laType la)
{
	if (la.type == MATRIX)
	{
		printf("Matrix /%.3f,\t%.3f,\t%.3f,\t%.3f\\\n"
			   "       |%.3f,\t%.3f,\t%.3f,\t%.3f|\n"
			   "       |%.3f,\t%.3f,\t%.3f,\t%.3f|\n"
			   "       \\%.3f,\t%.3f,\t%.3f,\t%.3f/\n",
			   la.a[0], la.a[1], la.a[2], la.a[3],
			   la.a[4], la.a[5], la.a[6], la.a[7],
			   la.a[8], la.a[9], la.a[10],la.a[11],
			   la.a[12],la.a[13],la.a[14],la.a[15]);
	}
	else if (la.type == VECTOR)
	{
		printf("LA 3D Vector (%.3f, %.3f, %.3f)\n",la.a[0],la.a[1],la.a[2]);
	}
	else if (la.type == 4)
	{
		printf("LA 4D Vector (%.3f, %.3f, %.3f, %.3f)\n",la.a[0],la.a[1],la.a[2],la.a[3]);
	}
	else
	{
		fprintf(stderr,"Wrong la type for print %d\n",la.type);
	}
}

laTypeS laTypeCastToSingle(laType* in)
{
	laTypeS result = {{in->a[0], in->a[1], in->a[2], in->a[3],
					   in->a[4], in->a[5], in->a[6], in->a[7],
					   in->a[8], in->a[9], in->a[10],in->a[11],
					   in->a[12],in->a[13],in->a[14],in->a[15]},in->type};
	return result;
}

laTypeD laTypeCastToDouble(laType* in)
{
	laTypeD result = {{in->a[0], in->a[1], in->a[2], in->a[3],
					   in->a[4], in->a[5], in->a[6], in->a[7],
					   in->a[8], in->a[9], in->a[10],in->a[11],
					   in->a[12],in->a[13],in->a[14],in->a[15]},in->type};
	return result;
}

laVector laTypeCastToVector(laType vec)
{
	laVector vector = {vec.a[0],vec.a[1],vec.a[2],vec.a[3]};
	return vector;
}
laMatrix laTypeCastToMatrix(laType mat)
{
	return *(laMatrix*)&mat;
}

laType Addf(laType a,float b)
{
	laType result;
	result.type = a.type;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		fprintf(stderr, "laType: invalid types for Addf\n");
		exit(-1);
	}
#endif

	for (unsigned i=0;i<a.type;i++)
	{
		result.a[i] = a.a[i] + b;
	}
	return result;
}

laType Subf(laType a,float b)
{
	laType result;

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

laType Mulf(laType a,float b)
{
	laType result = a;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		fprintf(stderr, "laType: invalid types for Mulf\n");
		exit(-1);
	}
#endif

	result.type = a.type;
	for (unsigned i=0;i<a.type;i++)
	{
		result.a[i] = a.a[i] * b;
	}
	return result;
}

laType Divf(laType a,float b)
{
	laType result;

#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		printf("Invalid types for Divf\n");
		exit(-1);
	}
#endif

	result.type = a.type;
	for (unsigned i=0;i<a.type;i++)
	{
		result.a[i] = a.a[i] / b;
	}
	return result;
}

laType Mul(laType a,laType b)
{
	laType result;
	result.type = 0;
	if (a.type==MATRIX && b.type==VECTOR)
	{
		result.type = VECTOR;
		result.a[0] = a.a[0]*b.a[0]+a.a[1]*b.a[1]+a.a[2]*b.a[2];
		result.a[1] = a.a[4]*b.a[0]+a.a[5]*b.a[1]+a.a[6]*b.a[2];
		result.a[2] = a.a[8]*b.a[0]+a.a[9]*b.a[1]+a.a[10]*b.a[2];
	}
	if (a.type==VECTOR && b.type==MATRIX)
	{
		result.type = VECTOR;
		result.a[0] = b.a[0]*a.a[0]+b.a[1]*a.a[1]+b.a[2] *a.a[2];
		result.a[1] = b.a[4]*a.a[0]+b.a[5]*a.a[1]+b.a[6] *a.a[2];
		result.a[2] = b.a[8]*a.a[0]+b.a[9]*a.a[1]+b.a[10]*a.a[2];
	}
	if (a.type==VECTOR4 && b.type==MATRIX)
	{
		result.type = VECTOR4;
		result.a[0] = b.a[0 ]*a.a[0]+b.a[1 ]*a.a[1]+b.a[2 ]*a.a[2]+b.a[3 ]*a.a[3];
		result.a[1] = b.a[4 ]*a.a[0]+b.a[5 ]*a.a[1]+b.a[6 ]*a.a[2]+b.a[7 ]*a.a[3];
		result.a[2] = b.a[8 ]*a.a[0]+b.a[9 ]*a.a[1]+b.a[10]*a.a[2]+b.a[11]*a.a[3];
		result.a[3] = b.a[12]*a.a[0]+b.a[13]*a.a[1]+b.a[14]*a.a[2]+b.a[15]*a.a[3];
	}
	if (a.type==MATRIX && b.type==MATRIX)
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
	if (!result.type)
	{
		printf("Invalid types for multiplication %d %d\n",a.type,b.type);
		exit(-1);
	}
#endif
	return result;
}

laType Mulmp(laType *a,laType *b)
{
	laType result;
	result.type = MATRIX;
	if (a->type==b->type && b->type==MATRIX)
	{
		result.a[0] = a->a[0]*b->a[0] + a->a[1]*b->a[4] + a->a[2]*b->a[8] + a->a[3]*b->a[12];
		result.a[1] = a->a[0]*b->a[1] + a->a[1]*b->a[5] + a->a[2]*b->a[9] + a->a[3]*b->a[13];
		result.a[2] = a->a[0]*b->a[2] + a->a[1]*b->a[6] + a->a[2]*b->a[10]+ a->a[3]*b->a[14];
		result.a[3] = a->a[0]*b->a[3] + a->a[1]*b->a[7] + a->a[2]*b->a[11]+ a->a[3]*b->a[15];

		result.a[4] = a->a[4]*b->a[0] + a->a[5]*b->a[4] + a->a[6]*b->a[8] + a->a[7]*b->a[12];
		result.a[5] = a->a[4]*b->a[1] + a->a[5]*b->a[5] + a->a[6]*b->a[9] + a->a[7]*b->a[13];
		result.a[6] = a->a[4]*b->a[2] + a->a[5]*b->a[6] + a->a[6]*b->a[10]+ a->a[7]*b->a[14];
		result.a[7] = a->a[4]*b->a[3] + a->a[5]*b->a[7] + a->a[6]*b->a[11]+ a->a[7]*b->a[15];

		result.a[8] = a->a[8]*b->a[0] + a->a[9]*b->a[4] + a->a[10]*b->a[8] + a->a[11]*b->a[12];
		result.a[9] = a->a[8]*b->a[1] + a->a[9]*b->a[5] + a->a[10]*b->a[9] + a->a[11]*b->a[13];
		result.a[10]= a->a[8]*b->a[2] + a->a[9]*b->a[6] + a->a[10]*b->a[10]+ a->a[11]*b->a[14];
		result.a[11]= a->a[8]*b->a[3] + a->a[9]*b->a[7] + a->a[10]*b->a[11]+ a->a[11]*b->a[15];

		result.a[12]= a->a[12]*b->a[0]+ a->a[13]*b->a[4]+ a->a[14]*b->a[8] + a->a[15]*b->a[12];
		result.a[13]= a->a[12]*b->a[1]+ a->a[13]*b->a[5]+ a->a[14]*b->a[9] + a->a[15]*b->a[13];
		result.a[14]= a->a[12]*b->a[2]+ a->a[13]*b->a[6]+ a->a[14]*b->a[10]+ a->a[15]*b->a[14];
		result.a[15]= a->a[12]*b->a[3]+ a->a[13]*b->a[7]+ a->a[14]*b->a[11]+ a->a[15]*b->a[15];
		return result;
	}
	else
	{
		fprintf(stderr,"laType error: invalid types\n");
		exit(-1);
	}
}


laType Mulmc(laType a,laType b)
{
	laType result;
	result.type = MATRIX;
	if (a.type==b.type && b.type==MATRIX)
	{
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
		return result;
	}
	else
	{
		fprintf(stderr,"laType error: invalid types for Mulmc (%d * %d)\n", a.type, b.type);
		exit(-1);
	}
}

laType Add(laType a,laType b)
{
	laType result;
	result.type = 0;
	if (a.type == b.type)
	{
		result.type = a.type;
		for (unsigned i=0;i<a.type;i++)
		{
			result.a[i] = a.a[i] + b.a[i];
		}
		return result;
	}
	else if (a.type==VECTOR && b.type==MATRIX16)
	{
		result = b;
		result.type = MATRIX16;
		result.a[3] = a.a[0] + b.a[3];
		result.a[7] = a.a[1] + b.a[7];
		result.a[11] = a.a[2] + b.a[11];
		return result;
	}
	else if (a.type==MATRIX16 && b.type==VECTOR)
	{
		result = a;
		result.type = MATRIX16;
		result.a[3]  = a.a[3]  + b.a[0];
		result.a[7]  = a.a[7]  + b.a[1];
		result.a[11] = a.a[11] + b.a[2];
		return result;
	}
#ifdef CHECK_TYPES
	if (!result.type)
	{
		printf("Invalid types for summa\n");
		exit(-1);
	}
#endif
	return result;
}

laType Sub(laType a,laType b)
{
	laType result;
	result.type = 0;
	if (a.type == b.type)
	{
		result.type = a.type;
		for (unsigned i=0;i<a.type;i++)
		{
			result.a[i] = a.a[i] - b.a[i];
		}
		return result;
	}
	else if (a.type==VECTOR && b.type==MATRIX16)
	{
		result = b;
		result.type = MATRIX16;
		result.a[3] = b.a[3] - a.a[0];
		result.a[6] = b.a[7] - a.a[1];
		result.a[9] = b.a[11] - a.a[2];
		return result;
	}
	else if (a.type==MATRIX16 && b.type==VECTOR)
	{
		result = a;
		result.type = MATRIX16;
		result.a[3] = -b.a[0] + a.a[3];
		result.a[7] = -b.a[1] + a.a[7];
		result.a[11]= -b.a[2] + a.a[11];
		return result;
	}
#ifdef CHECK_TYPES
	if (a.type!=3 && a.type!=4 && a.type!=9 && a.type!=16)
	{
		printf("Invalid types for subtraction\n");
		exit(-1);
	}
#endif
	return result;
}

laType Cross(laType a,laType b)
{
	laType result;
	result.type = 0;
	result.type = VECTOR;
	result.a[0] = a.a[1]*b.a[2] - b.a[1]*a.a[2];
	result.a[1] = -(a.a[0]*b.a[2] - b.a[0]*a.a[2]);
	result.a[2] = a.a[0]*b.a[1] - b.a[0]*a.a[1];
#ifdef CHECK_TYPES
	if (!result.type)
	{
		printf("Invalid types for cross\n");
		exit(-1);
	}
#endif
	return result;
}

laType Crossn(laType a,laType b)
{
	laType result;
	result.type = VECTOR;
	result.a[0] = a.a[1]*b.a[2] - b.a[1]*a.a[2];
	result.a[1] = -(a.a[0]*b.a[2] - b.a[0]*a.a[2]);
	result.a[2] = a.a[0]*b.a[1] - b.a[0]*a.a[1];
	float length = fiSqrt(result.a[0]*result.a[0]+result.a[1]*result.a[1]+result.a[2]*result.a[2]);
	result.a[0] *= length;
	result.a[1] *= length;
	result.a[2] *= length;
#ifdef CHECK_TYPES
	if (a.type!=VECTOR || b.type!=VECTOR)
	{
		printf("Invalid types for cross\n");
		exit(-1);
	}
#endif
	return result;
}

float Dot(laType a,laType b)
{
	if (a.type == b.type && a.type == VECTOR)
	{
		return a.a[0]*b.a[0] + a.a[1]*b.a[1] + a.a[2]*b.a[2];
	}

#ifdef CHECK_TYPES
	printf("Invalid types for dot\n");
	exit(-1);
#endif
}

float Dotn(laType a,laType b)
{
	if (a.type == b.type && a.type == VECTOR)
	{
		return (a.a[0]*b.a[0] + a.a[1]*b.a[1] + a.a[2]*b.a[2])
				*fiSqrt(a.a[0]*a.a[0]+a.a[1]*a.a[1]+a.a[2]*a.a[2])
				*fiSqrt(b.a[0]*b.a[0]+b.a[1]*b.a[1]+b.a[2]*b.a[2]);
	}
	if (a.type == b.type && a.type == VECTOR2)
	{
		return (a.a[0]*b.a[0] + a.a[1]*b.a[1])
				*fiSqrt(a.a[0]*a.a[0]+a.a[1]*a.a[1])
				*fiSqrt(b.a[0]*b.a[0]+b.a[1]*b.a[1]);
	}

#ifdef CHECK_TYPES
	printf("Invalid types for dot\n");
	exit(-1);
#endif
}

laType Perspective(float width,
					float height,
					float zfar,
					float znear,
					float angle)
{
	angle = 1.0/tan(angle/2.0/180.0*3.1415926535);
	laType result = {{angle/width*height,0.0,0.0,0.0,
					  0.0,angle,0.0,0.0,
					  0.0,0.0,-(znear+zfar)/(zfar-znear),-2.0*znear*zfar/(zfar-znear),
					  0.0,0.0,-1.0,0.0},MATRIX};
	return result;
}

laType Ortho(float size,
			 float zfar,
			 float znear)
{
	float dz = zfar-znear;
	laType result = {{2.0/size,	0.0,		0.0,		0.0,
					  0.0,		2.0/size,	0.0,		0.0,
					  0.0,		0.0,		-2.0/(dz),	(-dz)/(dz),
					  0.0,		0.0,		0.0,		1.0},MATRIX};
	return result;
}

laType RotationX(float angle)
{
	laType result;
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

laType RotationY(float angle)
{
	laType result;
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

laType RotationZ(float angle)
{
	laType result;
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

laType RotationXYZ(float x,float y,float z)
{
	return Mul(Mul(RotationX(x),RotationY(y)),RotationZ(z));
}

void RotateXYZ(laType* mat,float x,float y,float z)
{
	*mat = Mul(*mat,RotationXYZ(x,y,z));
}

void RotateXYZlocal(laType* mat,float x,float y,float z)
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
	*mat = Mul(*mat,RotationXYZ(x,y,z));
	mat->a[3] = px;
	mat->a[7] = py;
	mat->a[11]= pz;
}

void RotateXYZglobal(laType* mat,float x,float y,float z)
{
	float px = mat->a[3],py = mat->a[7],pz = mat->a[11];
	mat->a[3] = 0;
	mat->a[7] = 0;
	mat->a[11]= 0;
	*mat = Mul(RotationXYZ(x,y,z),*mat);
	mat->a[3] = px;
	mat->a[7] = py;
	mat->a[11]= pz;
}

void RotateByAxis(laType *mat,laType axis,float angle)
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
	*mat = Mul(Matrix3x3(ca+(1.0-ca)*x*x,  (1.0-ca)*x*y-sa*z,  (1-ca)*x*z+sa*y,
						(1.0-ca)*y*x+sa*z, ca+(1.0-ca)*y*y,    (1.0-ca)*y*z-sa*x,
						(1.0-ca)*z*x-sa*y, (1.0-ca)*z*y+sa*x,  ca+(1.0-ca)*z*z),*mat);
	mat->a[3] = axis.a[0];
	mat->a[7] = axis.a[1];
	mat->a[11]= axis.a[2];
}

void Translatel(laType* mat,float x,float y,float z)
{
	*mat = Mul(*mat,Add(Identity,Vector(x,y,z)));
}

void Translate(laType* mat,float x,float y,float z)
{
	*mat = Mul(*mat,Add(Identity,Vector(x,y,z)));
}

void Translateg(laType* mat,float x,float y,float z)
{
	*mat = Mul(Add(Identity,Vector(x,y,z)),*mat);
}

void SetPositiong(laType* mat,float x,float y,float z)
{
	mat->a[3] = 0;
	mat->a[7] = 0;
	mat->a[11] = 0;
	Translateg(mat,x,y,z);
}

laType Vector2(float x,float y)
{
	laType result;
	memset(&result,0,sizeof(result));
	result.type = VECTOR2;
	result.a[0] = x;
	result.a[1] = y;
	return result;
}

laType Vector(float x,float y,float z)
{
	laType result;
	result.type = VECTOR3;
	result.a[0] = x;
	result.a[1] = y;
	result.a[2] = z;
	return result;
}
laType Vector4(float x,float y,float z,float w)
{
	laType result;
	result.type = 4;
	result.a[0] = x;
	result.a[1] = y;
	result.a[2] = z;
	result.a[3] = w;
	return result;
}

laType Matrix3x3(float a,float b,float c,
				  float d,float e,float f,
				  float g,float h,float i)
{
	laType result = Identity;
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

laType Matrix4x4(laType vector1,
				 laType vector2,
				 laType vector3,
				 laType vector4)
{
	if (vector1.type!=4) fprintf(stderr,"Warning: 1 argument has wrong type\n");
	if (vector2.type!=4) fprintf(stderr,"Warning: 2 argument has wrong type\n");
	if (vector3.type!=4) fprintf(stderr,"Warning: 3 argument has wrong type\n");
	if (vector4.type!=4) fprintf(stderr,"Warning: 4 argument has wrong type\n");
	laType result = {
	{
			vector1.a[0],vector2.a[0],vector3.a[0],vector4.a[0],
			vector1.a[1],vector2.a[1],vector3.a[1],vector4.a[1],
			vector1.a[2],vector2.a[2],vector3.a[2],vector4.a[2],
			vector1.a[3],vector2.a[3],vector3.a[3],vector4.a[3]
	}, MATRIX};
	return result;
}

laType _iMatrix3x3(float a,float b,float c,
				  float d,float e,float f,
				  float g,float h,float i)
{
	laType result = Identity;
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

laType Scalef(laType a,float b)
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

void Normalize(laType* vector)
{
#ifdef CHECK_TYPES
	if (vector->type!=VECTOR)
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


void Transpose(laType* mat)
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

laType Transposed(laType mat)
{
	Transpose(&mat);
	return mat;
}

float Minor(laType mat,uint8_t index)
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

inline float Determinant(laType mat)
{
	if (mat.type!=MATRIX)
	{
		printf("Invalid type (%d) for determinant\n",mat.type);
		exit(-1);
	}
	return mat.a[A11]*Minor(mat,A11)-mat.a[A12]*Minor(mat,A12)+mat.a[A13]*Minor(mat,A13)-mat.a[A14]*Minor(mat,A14);
}

/*laType Inverted(laType mat)
{
	float det = Determinant(mat);
	laType connected = Identity;
	for (uint8_t i=0;i<16;i++)
	{
		connected.a[i] = Minor(mat,i)*(((~i ^ ~(i>>2))&1) ? -1.0 : 1.0) / det;
	}
	Transpose(&connected);

	return connected;
}*/
float laDet = 0.0f;
laType _Inverted(laType mat, char* filename, int line)
{
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
			fprintf(stderr, "Warning: laType Inverted() have zero determinant in %s:%d\n",filename, line);
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

laType Inverted(laType mat)
{
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

void IdentityArray3x3(float* mat)
{
	for (int o=0; o<9; o++)
	{
		mat[o] = (o%4)==0;
	}
}

void InvertArray3x3(float* result, float* mat)
{
	float(*m)[3];
	float out[3][3];

	m = (float(*)[3])mat;

	float a00 = m[0][0], a01 = m[0][1], a02 = m[0][2];
	float a10 = m[1][0], a11 = m[1][1], a12 = m[1][2];
	float a20 = m[2][0], a21 = m[2][1], a22 = m[2][2];

	float b01 = a22 * a11 - a12 * a21;
	float b11 = -a22 * a10 + a12 * a20;
	float b21 = a21 * a10 - a11 * a20;

	float det = a00 * b01 + a01 * b11 + a02 * b21;

	out[0][0] = b01;	out[0][1] = (-a22 * a01 + a02 * a21);	out[0][2] = a12 * a01 - a02 * a11;
	out[1][0] = b11;	out[1][1] = (a22 * a00 + a02 * a20);	out[1][2] = -a12 * a00 + a02 * a10;
	out[2][0] = b21;	out[2][1] = (-a21 * a00 + a01 * a20);	out[2][2] = a11 * a00 - a01 * a10;

	for (int i=0; i<9; i++)
	{
		result[i] = ((float*)out)[i] / det;
	}
}

void MulArrays3x3(float* result, float* mat1, float* mat2)
{
	float(*a)[3];
	float(*b)[3];
	float c[3][3];
	a = (float(*)[3])mat1;
	b = (float(*)[3])mat2;

	c[0][0] = a[0][0] * b[0][0] + a[0][1] * b[1][0] + a[0][2] * b[2][0];
	c[0][1] = a[0][0] * b[0][1] + a[0][1] * b[1][1] + a[0][2] * b[2][1];
	c[0][2] = a[0][0] * b[0][2] + a[0][1] * b[1][2] + a[0][2] * b[2][2];

	c[1][0] = a[1][0] * b[0][0] + a[1][1] * b[1][0] + a[1][2] * b[2][0];
	c[1][1] = a[1][0] * b[0][1] + a[1][1] * b[1][1] + a[1][2] * b[2][1];
	c[1][2] = a[1][0] * b[0][2] + a[1][1] * b[1][2] + a[1][2] * b[2][2];

	c[2][0] = a[2][0] * b[0][0] + a[2][1] * b[1][0] + a[2][2] * b[2][0];
	c[2][1] = a[2][0] * b[0][1] + a[2][1] * b[1][1] + a[2][2] * b[2][1];
	c[2][2] = a[2][0] * b[0][2] + a[2][1] * b[1][2] + a[2][2] * b[2][2];

	for (int i=0; i<9; i++)
	{
		result[i] = ((float*)c)[i];
	}
}

void MulVectorByMatrixArray3x3(float* result, float* vector, float* matrix)
{
	float(*mat)[3];
	mat = (float(*)[3])matrix;

	float x = vector[0] * mat[0][0] + vector[1] * mat[1][0] + vector[2] * mat[2][0];
	float y = vector[0] * mat[0][1] + vector[1] * mat[1][1] + vector[2] * mat[2][1];
	float w = vector[0] * mat[0][2] + vector[1] * mat[1][2] + vector[2] * mat[2][2];

	result[0] = x;
	result[1] = y;
	result[2] = w;
}

void MulMatrixByVectorArray3x3(float* result, float* matrix,float* vector)
{
	float(*mat)[3];
	mat = (float(*)[3])matrix;

	float x = vector[0] * mat[0][0] + vector[0] * mat[0][1] + vector[0] * mat[0][2];
	float y = vector[1] * mat[1][0] + vector[1] * mat[1][1] + vector[1] * mat[1][2];
	float w = vector[2] * mat[2][0] + vector[2] * mat[2][1] + vector[2] * mat[2][2];

	result[0] = x;
	result[1] = y;
	result[2] = w;
}

void RotationArray3x3(float* result, float x)
{
	float(*a)[3];
	a = (float(*)[3])result;

	a[0][0] = cosf(x);
	a[0][1] =-sinf(x);
	a[0][2] = 0.0;

	a[1][0] = sinf(x);
	a[1][1] = cosf(x);
	a[1][2] = 0.0;

	a[2][0] = 0.0;
	a[2][1] = 0.0;
	a[2][2] = 1.0;
}

float RotationFromArray3x3(float* matrix)
{
	return atan2f(matrix[3], matrix[0]);
}

laType InvertedFast(laType mat)
{
	laType rot = GetOrientationTransposed(mat);
	laType pos = Mul(rot,mat);
	rot.a[3] = -pos.a[3];
	rot.a[7] = -pos.a[7];
	rot.a[11]= -pos.a[11];
	return rot;
}

laType GetOrientation(laType mat)
{
	return Matrix3x3(mat.a[0],mat.a[1],mat.a[2],mat.a[4],mat.a[5],mat.a[6],mat.a[8],mat.a[9],mat.a[10]);
}

laType GetOrientationTransposed(laType mat)
{
	return _iMatrix3x3(mat.a[0],mat.a[4],mat.a[8],
					 mat.a[1],mat.a[5],mat.a[9],
					 mat.a[2],mat.a[6],mat.a[10]);
}

laType GetPosition(laType mat)
{
	//mat = Mul(GetOrientationTransposed(mat),mat);
	laType vector = {{mat.a[3],mat.a[7],mat.a[11]},VECTOR};
	return vector;
}

laType GetPositionMatrix(laType mat)
{
	mat.a[9] = mat.a[8] = mat.a[6] = mat.a[5] = mat.a[2] = mat.a[1] = 0.0;
	mat.a[10] = mat.a[5] = mat.a[0] = 1.0;
	return mat;
	//return Mul(GetOrientationTransposed(mat),mat);
}

laType GetVectorTo(laType mat1,laType mat2)
{
	//return GetPosition(Mul(Inverted(mat1),mat2));
	return Sub(GetPosition(mat2),GetPosition(mat1));
}

laType GetNVectorTo(laType mat1,laType mat2)
{
	mat1 = Sub(GetPosition(mat2),GetPosition(mat1));
	float length = fiSqrt(mat1.a[0]*mat1.a[0]+mat1.a[1]*mat1.a[1]+mat1.a[2]*mat1.a[2]);
	mat1.a[0] *= length;
	mat1.a[1] *= length;
	mat1.a[2] *= length;
	mat1.type = VECTOR;
	return mat1;
}

float Length(laType vector)
{
	if (vector.type!=VECTOR)
	{
		printf("Invalid type (%d) for length\n",vector.type);
		exit(-1);
	}
	return fsqrt(vector.a[0]*vector.a[0]+vector.a[1]*vector.a[1]+vector.a[2]*vector.a[2]);
}

void SetCameraDirection(laType *mat1,laType lookaxis)
{
	/*if (upaxis.type!=VECTOR)
	{
		fprintf(stderr,"SetMatrixDirection argument upaxis should be a VECTOR\n");
		exit(-1);
	}*/
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
	/*lookaxis.a[0] = -lookaxis.a[0];
	lookaxis.a[1] = -lookaxis.a[1];
	lookaxis.a[2] = -lookaxis.a[2];*/
	//laType upDirection = Crossn(lookaxis,Vector(0.0,0.0,1.0));
	laType rightDirection = Crossn(lookaxis,Vector(0.0,0.0,-1.0));
	laType upDirection = Crossn(lookaxis,rightDirection);
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

laType LookAt(laType mat1,laType mat2,laAxis upaxis,laAxis lookaxis)
{
	laType look_axes[] = {
			{{ 1.0, 0.0, 0.0},VECTOR}, {{0.0, 1.0, 0.0},VECTOR}, {{0.0, 0.0, 1.0},VECTOR},
			{{-1.0, 0.0, 0.0},VECTOR}, {{0.0,-1.0, 0.0},VECTOR}, {{0.0, 0.0,-1.0},VECTOR}
	};

	laType Eye = GetPosition(mat1);
	laType At  = GetPosition(mat2);
	laType zaxis = Sub(Eye, At); Normalize(&zaxis);
	laType Up = look_axes[upaxis];//	Up.a[upaxis%3] = 1.0 - 2.0*(upaxis>laZ);
	laType xaxis = Crossn(Up, zaxis);
	laType yaxis = Crossn(zaxis, xaxis);

	switch (lookaxis)
	{
		case laZ : {
			laType result = {{
			 xaxis.a[0],    yaxis.a[0],    zaxis.a[0],   mat1.a[3],
			 xaxis.a[1],    yaxis.a[1],    zaxis.a[1],   mat1.a[7],
			 xaxis.a[2],    yaxis.a[2],    zaxis.a[2],   mat1.a[11],
			 0.0,           0.0,        0.0,             1.0
			}, 16};
			return result;
		}
		case laX : {
			laType result = {{
			 zaxis.a[0],    xaxis.a[0],    yaxis.a[0],   mat1.a[3],
			 zaxis.a[1],    xaxis.a[1],    yaxis.a[1],   mat1.a[7],
			 zaxis.a[2],    xaxis.a[2],    yaxis.a[2],   mat1.a[11],
			 0.0,           0.0,        0.0,             1.0
			}, 16};
			return result;
		}
		case laY : {
			laType result = {{
			 yaxis.a[0],    zaxis.a[0],    xaxis.a[0],   mat1.a[3],
			 yaxis.a[1],    zaxis.a[1],    xaxis.a[1],   mat1.a[7],
			 yaxis.a[2],    zaxis.a[2],    xaxis.a[2],   mat1.a[11],
			 0.0,           0.0,        0.0,             1.0
			}, 16};
			return result;
		}
		default : return Identity;
	}
	return Identity;
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

laType Interpolate(laType mat1,laType mat2,float coeff)
{
    if (coeff>1.0) coeff = 1.0;
    if (coeff<0.0) coeff = 0.0;
    laType result;
    result = Add(Mulf(mat1,1.0-coeff),Mulf(mat2,coeff));

    laType v1 = Vector(result.a[0], result.a[4], result.a[ 8]);
    laType v2 = Vector(result.a[1], result.a[5], result.a[ 9]);
    laType v3 = Vector(result.a[2], result.a[6], result.a[10]);

    v3 = Crossn(v1,v2);
    v2 = Crossn(v3,v1);
    v1 = Crossn(v2,v3);

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

laType InterpolateIn(laType mat1,laType mat2,float coeff)
{
	/*laType result = Mulmc( mat1, Interpolate(Identity, Mulmc(mat2,Inverted(mat1)), coeff) );
	result.a[3] = mat1.a[3]*(1.0-coeff) + mat2.a[3]*coeff;
	result.a[7] = mat1.a[7]*(1.0-coeff) + mat2.a[7]*coeff;
	result.a[11] = mat1.a[11]*(1.0-coeff) + mat2.a[11]*coeff;*/
	return Add(Mulf(mat1,1.0-coeff),Mulf(mat2,coeff));
}
/*
11 12 13 14
21 22 23 24
31 32 33 34
41 42 43 44
*/

laType ToEuler(laType in_mat)
{
	laType result = {{0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0},VECTOR};
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

float laTypeGetItem(laType lat,int i)
{
	return lat.a[i];
}
void laTypeSetItem(laType *lat,int i,float val)
{
	lat->a[i] = val;
}
int laTypeGetType(laType lat)
{
	return lat.type;
}
