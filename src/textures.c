/*
 * textures.c
 *
 *  Created on: 21 янв. 2018 г.
 *      Author: ivan
 */

#ifndef TEXTURES_C_
#define TEXTURES_C_

#include "engine.h"
#define DDS_HEADER_SIZE 128
#define DDS_SIGNATURE    0x20534444 // "DDS "
#define FORMAT_CODE_DXT1 0x31545844 // "DXT1"
#define FORMAT_CODE_DXT3 0x33545844 // "DXT3"
#define FORMAT_CODE_DXT5 0x35545844 // "DXT5"

void mulmat(float *a, float *b, float *c, int size)
{
	for (uint32_t i=0;i<size;i++)
	{
		float val = 0.0;
		for (uint32_t j=0;j<size;j++)
		{
			val += a[j] * b[j*size + i];
		}
		c[i] = val;
	}
}

void transpose(float *a,uint16_t size)
{
	for (uint16_t i=0;i<size;i++)
	{
		for (uint16_t j=0;j<i;j++)
		{
			float w = a[i*size + j];
			a[i*size + j] = a[j*size + i];
			a[j*size + i] = w;
		}
	}
}

float clamp(float val, float min, float max)
{
	val = val>max ? max : val;
	val = val<min ? min : val;
	return val;
}

static int sCompressedTextureArrayOpen(DDS_DATA *file, char *name)
{
	FILE* fp = fopen(name,"rb");
	if (!fp)
	{
		return 1;
	}

	fseek(fp,0,SEEK_END);
	size_s fsize = ftell(fp);
	fseek(fp,0,SEEK_SET);

	file->dataPtr = sMalloc(fsize);
	readf(file->dataPtr,fsize,1,fp);
	fclose(fp);

	file->signature    = *(uint32_t*)(file->dataPtr);
	file->height       = *(uint32_t*)(file->dataPtr+12);
	file->width        = *(uint32_t*)(file->dataPtr+16);
	file->mipMapNumber = *(uint32_t*)(file->dataPtr+28);
	file->formatCode   = *(uint32_t*)(file->dataPtr+84);
	file->offset 	  =  DDS_HEADER_SIZE;

	switch(file->formatCode)
	{
	case FORMAT_CODE_DXT1:
		file->format = GL_COMPRESSED_RGBA_S3TC_DXT1_EXT;
		break;
	case FORMAT_CODE_DXT3:
		file->format = GL_COMPRESSED_RGBA_S3TC_DXT3_EXT;
		break;
	case FORMAT_CODE_DXT5:
		file->format = GL_COMPRESSED_RGBA_S3TC_DXT5_EXT;
		break;
	default:
		fprintf(stderr,"Unknown format 0x%08X of %s\n",file->formatCode,name); return 2;
	}
	file->blockSize = (file->format == GL_COMPRESSED_RGBA_S3TC_DXT1_EXT) ? 8 : 16;
	return 0;
}

static void sCompressedTextureArrayClose(DDS_DATA *file)
{
	sFree(file->dataPtr);
	file->dataPtr = 0;
}

static void sCompressedCubemapSide(DDS_DATA *file,int type)
{
	uint32_t width = file->width;
	uint32_t height = file->height;

	for (uint32_t level = 0; level < file->mipMapNumber; ++level)
	{
		uint32_t size = ((width+3)/4)*((height+3)/4)*file->blockSize;

		glCompressedTexImage2D(type, level, file->format, width, height, 0, size, file->dataPtr + file->offset);
		width = width > 1 ? width >> 1 : 1;
		height = height > 1 ? height >> 1 : 1;
		file->offset += size;
	}
}
/*
static void sCompressedSubimage(DDS_DATA *file,int number)
{
	uint32_t width = file->width;
	uint32_t height = file->height;

	for (uint32_t level = 0; level < file->mipMapNumber; ++level)
	{
		uint32_t size = ((width+3)/4)*((height+3)/4)*file->blockSize;

		glc(glCompressedTexSubImage3D(GL_TEXTURE_2D_ARRAY, level, 0, 0, number, width, height, 1, file->format, size, file->dataPtr + file->offset));
		width = width > 1 ? width >> 1 : 1;
		height = height > 1 ? height >> 1 : 1;
		file->offset += size;
	}
}
*/
void sTextureGenerateBlueNoise(sTexture* texture)
{
	srand(3475983);
	int n=texture->width;
	int count = 3*64;

	glGenTextures(1,&texture->ID);
	glBindTexture(GL_TEXTURE_2D_ARRAY,texture->ID);
	glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
	glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
	glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_WRAP_S, GL_REPEAT);
	glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_WRAP_T, GL_REPEAT);

	glc(glTexStorage3D(GL_TEXTURE_2D_ARRAY, 1, GL_R8, n, n, count));

	float DCT_mat[n][n];
	float noise1[n][n];
	float noise2[n][n];
	uint8_t noise_tex[n][n];
	float step = 1.0/n*3.1415926535;
	for (uint32_t img=0;img<count;img++)
	{
		//printf("Noise sample #%ud\n",img);
		for (uint32_t i=0;i<n;i++)
		{
			for (uint32_t j=0;j<n;j++)
			{
				DCT_mat[i][j] = cosf(j*step * i);

				float x = (float)i/n;
				float y = (float)j/n;
				float mul = pow(x*x + y*y, 2.0) * 0.2;

				mul = mul>1.0 ? 1.0 : mul;
				mul = mul<0.0 ? 0.0 : mul;
				float val = (((double)rand()/(double)RAND_MAX)-0.5)*2.;
				noise1[i][j] = val * 1.0 * mul;
			}
		}

		for (uint32_t i=0;i<n;i++)
		{
			mulmat(noise1[i],(float*)DCT_mat,noise2[i],n);
		}
		transpose((float*)noise2,n);
		for (uint32_t i=0;i<n;i++)
		{
			mulmat(noise2[i],(float*)DCT_mat,noise1[i],n);
			for (uint32_t j=0;j<n;j++)
			{
				noise1[i][j]/= n;
				noise_tex[i][j] = clamp(noise1[i][j]*0.5+0.5,0.0,1.0)*255.0;
			}
		}
		glc(glTexSubImage3D(GL_TEXTURE_2D_ARRAY, 0,
					0, 0, img,
					n,n, 1,
					GL_RED,
					GL_UNSIGNED_BYTE,
					noise_tex)
			);
	}
}

int sTextureLoadDDSFromString(sTexture* texture, char* content)
{
	glGetError();
	void* dataPtr = content;

	uint32_t signature    = *(uint32_t*)(dataPtr);
	uint32_t height       = *(uint32_t*)(dataPtr+12);
	uint32_t width        = *(uint32_t*)(dataPtr+16);
	uint32_t mipMapNumber = *(uint32_t*)(dataPtr+28);
	uint32_t formatCode   = *(uint32_t*)(dataPtr+84);

	if (signature!=DDS_SIGNATURE)
	{
		//fprintf(stderr,"%s is not a DDS file\n",name);
		return 2;
	}

	texture->width = width;
	texture->height = height;
	//printf("%hux%hu\n",width,height);

	uint32_t format;
	switch(formatCode)
	{
	case FORMAT_CODE_DXT1:
		format = GL_COMPRESSED_RGBA_S3TC_DXT1_EXT;
		break;
	case FORMAT_CODE_DXT3:
		format = GL_COMPRESSED_RGBA_S3TC_DXT3_EXT;
		break;
	case FORMAT_CODE_DXT5:
		format = GL_COMPRESSED_RGBA_S3TC_DXT5_EXT;
		break;
	default:
		return 1;
	}
	uint32_t blockSize = (format == GL_COMPRESSED_RGBA_S3TC_DXT1_EXT) ? 8 : 16;
	uint32_t offset = DDS_HEADER_SIZE;

	//puts("glGenTextures");
	glc(glGenTextures(1,&texture->ID));
	//puts("glBindTexture");
	glc(glBindTexture(GL_TEXTURE_2D,texture->ID));
	for (uint32_t level = 0; level < mipMapNumber; ++level)
	{
		//printf("%d texture lod\n", level);
		uint32_t size = ((width+3)/4)*((height+3)/4)*blockSize;

		glc(glCompressedTexImage2D(GL_TEXTURE_2D, level, format, width, height, 0, size, dataPtr + offset));
		width = width > 1 ? width >> 1 : 1;
		height = height > 1 ? height >> 1 : 1;
		offset += size;
	}
	GLenum err = glGetError();
	if (err!=GL_NO_ERROR)
	{
		fprintf(stderr,"Texture error %d\n", err);
		//exit(-1);
	}
	//glGenerateMipmap(type);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 16);
	//puts("glTexParameteri");

	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR));

	//puts("glBindTexture");
	glc(glBindTexture(GL_TEXTURE_2D, 0));
	return 0;
}

int sTextureLoadDDS(sTexture* texture,char* name)
{
	glGetError();
	size_s fsize;
	void* dataPtr;
	FILE* fp = fopen(name,"rb");
	if (!fp)
	{
		//fprintf(stderr,"%s not such file\n",name);
		return 1;
	}

	fseek(fp,0,SEEK_END);
	fsize = ftell(fp);
	fseek(fp,0,SEEK_SET);

	dataPtr = sMalloc(fsize);
	readf(dataPtr,fsize,1,fp);
	uint32_t signature    = *(uint32_t*)(dataPtr);
	uint32_t height       = *(uint32_t*)(dataPtr+12);
	uint32_t width        = *(uint32_t*)(dataPtr+16);
	uint32_t mipMapNumber = *(uint32_t*)(dataPtr+28);
	uint32_t formatCode   = *(uint32_t*)(dataPtr+84);

	if (signature!=DDS_SIGNATURE)
	{
		//fprintf(stderr,"%s is not a DDS file\n",name);
		return 2;
	}

	texture->width = width;
	texture->height = height;
	//printf("%hux%hu\n",width,height);

	uint32_t format;
	switch(formatCode)
	{
	case FORMAT_CODE_DXT1:
		format = GL_COMPRESSED_RGBA_S3TC_DXT1_EXT;
		break;
	case FORMAT_CODE_DXT3:
		format = GL_COMPRESSED_RGBA_S3TC_DXT3_EXT;
		break;
	case FORMAT_CODE_DXT5:
		format = GL_COMPRESSED_RGBA_S3TC_DXT5_EXT;
		break;
	default:
		fprintf(stderr,"Unknown format 0x%08X of %s\n",formatCode,name); exit(-1);
	}
	uint32_t blockSize = (format == GL_COMPRESSED_RGBA_S3TC_DXT1_EXT) ? 8 : 16;
	uint32_t offset = DDS_HEADER_SIZE;

	//puts("glGenTextures");
	glc(glGenTextures(1,&texture->ID));
	//puts("glBindTexture");
	glc(glBindTexture(GL_TEXTURE_2D,texture->ID));
	for (uint32_t level = 0; level < mipMapNumber; ++level)
	{
		//printf("%d texture lod\n", level);
		uint32_t size = ((width+3)/4)*((height+3)/4)*blockSize;

		glc(glCompressedTexImage2D(GL_TEXTURE_2D, level, format, width, height, 0, size, dataPtr + offset));
		width = width > 1 ? width >> 1 : 1;
		height = height > 1 ? height >> 1 : 1;
		offset += size;
	}
	GLenum err = glGetError();
	if (err!=GL_NO_ERROR)
	{
		fprintf(stderr,"Texture %s error %d\n",name,err);
		//exit(-1);
	}
	//glGenerateMipmap(type);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 16);
	//puts("glTexParameteri");

	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR));

	//puts("glBindTexture");
	glc(glBindTexture(GL_TEXTURE_2D, 0));
	//puts("sFree");
	sFree(dataPtr);
	//puts("fclose");
	fclose(fp);
	return 0;
}


int sTextureLoadCubemap(sTexture* texture,char* name)
{
	glGetError();
	DDS_DATA image;
	glGenTextures(1,&texture->ID);
	strcpy(texture->name,"cbm");
	texture->hash = S_Name2hash(texture->name);
	glc(glEnable(GL_TEXTURE_CUBE_MAP));
	glc(glBindTexture(GL_TEXTURE_CUBE_MAP,texture->ID));

	int result = sCompressedTextureArrayOpen(&image,name);
	if (result)
	{
		return result;
	}
	glc(sCompressedCubemapSide(&image,GL_TEXTURE_CUBE_MAP_POSITIVE_X));
	glc(sCompressedCubemapSide(&image,GL_TEXTURE_CUBE_MAP_NEGATIVE_X));
	glc(sCompressedCubemapSide(&image,GL_TEXTURE_CUBE_MAP_POSITIVE_Y));
	glc(sCompressedCubemapSide(&image,GL_TEXTURE_CUBE_MAP_NEGATIVE_Y));
	glc(sCompressedCubemapSide(&image,GL_TEXTURE_CUBE_MAP_POSITIVE_Z));
	glc(sCompressedCubemapSide(&image,GL_TEXTURE_CUBE_MAP_NEGATIVE_Z));
	glc(sCompressedTextureArrayClose(&image));

	glc(glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
	glc(glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
	glc(glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE));
	glc(glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE));
	glc(glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_R, GL_CLAMP_TO_EDGE));

	return 0;
}


void sTextureFree(sTexture *texture)
{
	glDeleteTextures(1,&texture->ID);
}

#endif /* TEXTURES_C_ */
