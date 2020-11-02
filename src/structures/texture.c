/*
 * texture.c
 *
 *  Created on: 10 июля 2020 г.
 *      Author: Ivan G
 */

#include "structures/texture.h"
#include "io.h"
#define STB_IMAGE_IMPLEMENTATION
#include <stb/stb_image.h>
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb/stb_image_write.h>
#include <ctype.h>

#define DDS_HEADER_SIZE 128
#define DDS_SIGNATURE    0x20534444 // "DDS "
#define FORMAT_CODE_DXT1 0x31545844 // "DXT1"
#define FORMAT_CODE_DXT3 0x33545844 // "DXT3"
#define FORMAT_CODE_DXT5 0x35545844 // "DXT5"

#ifdef __cplusplus
extern "C" {
#endif

static sTextureID* textures = 0;

static int sCompressedTextureArrayOpen(DDS_DATA *file, const char *name)
{
	//if (!file->dataPtr)
	{
		FILE* fp = fopen(name,"rb");
		if (!fp)
		{
			return 1;
		}
		size_t fsize = sizef(fp);
		file->dataPtr = sMalloc(fsize);
		readf(file->dataPtr,fsize,1,fp);
		fclose(fp);
	}

	file->signature    = *(uint32_t*)((uintptr_t)file->dataPtr);
	file->height       = *(uint32_t*)((uintptr_t)file->dataPtr+12);
	file->width        = *(uint32_t*)((uintptr_t)file->dataPtr+16);
	file->mipMapNumber = *(uint32_t*)((uintptr_t)file->dataPtr+28);
	file->formatCode   = *(uint32_t*)((uintptr_t)file->dataPtr+84);
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
	sDelete(file->dataPtr);
	file->dataPtr = 0;
}

static void sCompressedCubemapSide(DDS_DATA *file,int type)
{
	uint32_t width = file->width;
	uint32_t height = file->height;

	for (uint32_t level = 0; level < file->mipMapNumber; ++level)
	{
		uint32_t size = ((width+3)/4)*((height+3)/4)*file->blockSize;
		glCompressedTexImage2D(type, level, file->format, width, height, 0, size, (void*)((uintptr_t)file->dataPtr + file->offset));
		width = width > 1 ? width >> 1 : 1;
		height = height > 1 ? height >> 1 : 1;
		file->offset += size;
	}
}

sTextureID sTextureCreateEmpty(char* name)
{
    sTextureID tex = sNew(sTexture);
    strcpy(tex->name, name);
    sListPushBack(textures, tex);
    return tex;
}

sTextureID sTextureCreate2D(char* name, uint16_t width, uint16_t height, sTexturePixFmt pix_fmt, bool filtering, bool mipmaps, void* data)
{
	sTextureID texture = sTextureCreateEmpty(name);
	
	GLint internal_format = sTextureFormatTable[pix_fmt][1];
	GLint format = sTextureFormatTable[pix_fmt][0];
	GLint type = sTextureFormatTable[pix_fmt][2];
	glc(glGenTextures(1, &texture->ID));
	glc(glBindTexture(GL_TEXTURE_2D, texture->ID));
	glc(glTexImage2D(GL_TEXTURE_2D, 0,internal_format, width, height, 0,format, type, (const void*)data));
	if (mipmaps)
	{
		glc(glGenerateMipmap(GL_TEXTURE_2D));
		glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
	}
	else
	{
		glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filtering ? GL_LINEAR : GL_NEAREST));
	}
	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filtering ? GL_LINEAR : GL_NEAREST));
	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE));
	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE));
	texture->width  = width;
	texture->height = height;
	texture->type = GL_TEXTURE_2D;
	texture->color_format = pix_fmt;
	return texture;
}

void sTextureSetTiling(sTextureID texture, sTextureSamplingMode tiling)
{
	glBindTexture(texture->type, texture->ID);
	switch (tiling) {
		case sTextureRepeat : 
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT));
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT));
			break;
		case sTextureRepeatMirror : 
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_MIRRORED_REPEAT));
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_MIRRORED_REPEAT));
			break;
		case sTextureClampEdge : 
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE));
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE));
			break;
		case sTextureClampBlack : 
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_BORDER));
			glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_BORDER));
			break;
	}
}

sTextureID sTextureCreateCubemap(char* name, uint16_t width, uint16_t height, sTexturePixFmt pix_fmt, bool filtering, bool mipmaps)
{
	sTextureID texture = sTextureCreateEmpty(name);
	
	GLint internal_format = sTextureFormatTable[pix_fmt][1];
	GLint format = sTextureFormatTable[pix_fmt][0];
	GLint type = sTextureFormatTable[pix_fmt][2];
	glGenTextures(1, &texture->ID);
	glBindTexture(GL_TEXTURE_CUBE_MAP, texture->ID);
	for (int i=0; i<6; i++)
	{
		glTexImage2D(GL_TEXTURE_CUBE_MAP_POSITIVE_X + i, 0,internal_format, width, height, 0,format, type, 0);
	}
	if (mipmaps)
	{
		glGenerateMipmap(GL_TEXTURE_CUBE_MAP);
		glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR);
	}
	else
	{
		glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MIN_FILTER, filtering ? GL_LINEAR : GL_NEAREST);
	}
	glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MAG_FILTER, filtering ? GL_LINEAR : GL_NEAREST);
	glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_R, GL_CLAMP_TO_EDGE);
	glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
	glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
	texture->width  = width;
	texture->height = height;
	texture->type = GL_TEXTURE_CUBE_MAP;
	texture->color_format = pix_fmt;
	return texture;
}

/*
GL_TEXTURE_CUBE_MAP_POSITIVE_X 0x8515
GL_TEXTURE_CUBE_MAP_NEGATIVE_X 0x8516
GL_TEXTURE_CUBE_MAP_POSITIVE_Y 0x8517
GL_TEXTURE_CUBE_MAP_NEGATIVE_Y 0x8518
GL_TEXTURE_CUBE_MAP_POSITIVE_Z 0x8519
GL_TEXTURE_CUBE_MAP_NEGATIVE_Z 0x851A
*/

void sTextureCubeSplit(sTextureID cubemap)
{
	if (cubemap->type != GL_TEXTURE_CUBE_MAP) 
	{
		printf("(%d) Тип текстуры не является GL_TEXTURE_CUBE_MAP\n", cubemap->type);
		return;
	}
	char* side_names[6] = { (char*)"+X", (char*)"-X", (char*)"+Y", (char*)"-Y", (char*)"+Z", (char*)"-Z" };
	for (int i=0; i<6; i++)
	{
		sTextureID side = sTextureCreateEmpty(cubemap->name);
		memcpy(side->name+strlen(side->name), side_names[i], 2);
		side->width = cubemap->width;
		side->height = cubemap->height;
		side->ID = cubemap->ID;
		side->color_format = cubemap->color_format;
		side->type = GL_TEXTURE_CUBE_MAP_POSITIVE_X + i;
		side->parent = cubemap;
		sListPushBack(cubemap->sides, side);
	}
}

void sTextureAddFramebufferUser(sTextureID texture, sFrameBufferID framebuffer)
{
	if (sListIndexOf(texture->framebuffer_users, framebuffer)==MAX_INDEX)
	{
		sListPushBack(texture->framebuffer_users, framebuffer);
	}
}

void sTextureAddMaterialUser(sTextureID texture, sMaterialID material)
{
	if (sListIndexOf(texture->material_users, material)==MAX_INDEX)
	{
		sListPushBack(texture->material_users, material);
	}
}

void sTextureRemoveFramebufferUser(sTextureID texture, sFrameBufferID framebuffer)
{
	sListPopItem(texture->framebuffer_users, framebuffer);
	sListPopItem(framebuffer->color_render_targets, texture);
}

void sTextureRemoveMaterialUser(sTextureID texture, sMaterialID material)
{
	if (!texture) return;
	sListPopItem(texture->material_users, material);
	sMaterialDetachTexture(material, texture);
}

void sTextureRemoveUsers(sTextureID texture)
{
	while (sListGetSize(texture->material_users)) {
		sTextureRemoveMaterialUser(texture, texture->material_users[0]);
	}
	while (sListGetSize(texture->framebuffer_users)) {
		sTextureRemoveFramebufferUser(texture, texture->framebuffer_users[0]);
	}
}

void sTextureDelete(sTextureID texture)
{
	sTextureRemoveUsers(texture);
	if (texture->parent) {
		sListPopItem(texture->parent, texture);
	} else {
		size_t sides_count = sListGetSize(texture->sides);
		sTextureID sides[sides_count];
		for (size_t i=0; i<sides_count; i++) { sides[i] = texture->sides[i]; }
		for (size_t i=0; i<sides_count; i++) { sTextureDelete(sides[i]); }
		glDeleteTextures(1,&texture->ID);
	}
	sDelete(texture->sides);
	sDelete(texture->data);
    sListPopItem(textures, texture);
    sDelete(texture->material_users);
    sDelete(texture->framebuffer_users);
	sDelete(texture);
}

sTextureID sTextureLoadDDSFromMem(char* name, void* dataPtr)
{
	glGetError();
    sTextureID texture = sTextureCreateEmpty(name);
	uint32_t signature    = *(uint32_t*)(dataPtr);
	uint32_t height       = *(uint32_t*)((uintptr_t)dataPtr+12);
	uint32_t width        = *(uint32_t*)((uintptr_t)dataPtr+16);
	uint32_t mipMapNumber = *(uint32_t*)((uintptr_t)dataPtr+28);
	uint32_t formatCode   = *(uint32_t*)((uintptr_t)dataPtr+84);

	if (signature!=DDS_SIGNATURE)
	{
		fprintf(stderr, "Texture \"%s\" is not DDS\n", name);
		return 0;
	}

	texture->width = width;
	texture->height = height;

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
		fprintf(stderr, "Texture \"%s\" have unknown DDS format\n", name);
		return 0;
	}
	uint32_t blockSize = (format == GL_COMPRESSED_RGBA_S3TC_DXT1_EXT) ? 8 : 16;
	uint32_t offset = DDS_HEADER_SIZE;

	if (!texture->ID)
		glc(glGenTextures(1,&texture->ID));
	
	glc(glBindTexture(GL_TEXTURE_2D,texture->ID));
	for (uint32_t level = 0; level < mipMapNumber; ++level)
	{
		uint32_t size = ((width+3)/4)*((height+3)/4)*blockSize;

		glc(glCompressedTexImage2D(GL_TEXTURE_2D, level, format, width, height, 0, size, (void*)((uintptr_t)dataPtr + offset)));
		width = width > 1 ? width >> 1 : 1;
		height = height > 1 ? height >> 1 : 1;
		offset += size;
	}
	GLenum err = glGetError();
	if (err!=GL_NO_ERROR)
	{
		fprintf(stderr,"Texture error %d\n", err);
        return 0;
	}
	
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, 16);

	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR));

	glc(glBindTexture(GL_TEXTURE_2D, 0));

	texture->type = GL_TEXTURE_2D;
	texture->color_format = format;

	return texture;
}

sTextureID sTextureLoadDDS(char* name)
{
	glGetError();
	size_t fsize;
	void* dataPtr;
	FILE* fp = fopen(name,"rb");
	if (!fp)
	{
		fprintf(stderr,"%s no such file\n",name);
		return 0;
	}

	fsize = sizef(fp);

	dataPtr = sMalloc(fsize);
	readf(dataPtr,fsize,1,fp);
    
	sTextureID result = sTextureLoadDDSFromMem(name, dataPtr);
	sFree(dataPtr);
	return result;
}

static int load_cubemap(sTexture* texture, const char* name)
{
	glGetError();
	DDS_DATA image;
	strcpy(texture->name, name);
	glc(glGenTextures(1,&texture->ID));
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
	
	texture->color_format = image.format;
	texture->type = GL_TEXTURE_CUBE_MAP;
	sDelete(texture->data);
	texture->data = 0;

	return 0;
}

sTextureID sTextureLoadDDSCubemap(const char* name)
{
	sTextureID texture;
	texture = sTextureCreateEmpty((char*)name);
	switch (load_cubemap(texture, name)) {
		case 1 : fprintf(stderr, "Texture \"%s\" not found\n", name); break;
		case 2 : fprintf(stderr, "Texture \"%s\" is not DirectDraw Surface\n", name); break;
	}
	return texture;
}

static void mulmat(float *a, float *b, float *c, uint32_t size)
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

void gen_dct_mat(float* arr, uint16_t size)
{
	float (*DCT_mat)[size] = (float(*)[size])arr;

	float step = 1.0/size*3.1415926535;

	for (uint32_t i=0;i<size;i++)
	{
		for (uint32_t j=0;j<size;j++)
		{
			DCT_mat[i][j] = cosf((j+1)*step * (i+1));
		}
	}
}

void gen_fa_mat(float* arr, uint16_t size, float(*fu)(float,float))
{
	float (*mat)[size] = (float(*)[size])arr;

	if (fu) {
		for (uint16_t i=0; i<size; i++) {
			for (uint16_t j=0; j<size; j++) {
				float val = (double)rand()/(double)RAND_MAX * 2.0 - 1.0;
				mat[i][j] = val*fu((float)i/size, (float)j/size);
			}
		}
	} else {
		for (uint16_t i=0; i<size; i++) {
			for (uint16_t j=0; j<size; j++) {
				float val = (double)rand()/(double)RAND_MAX * 2.0 - 1.0;
				mat[i][j] = val;
			}
		}
	}
}
#define MAX2(a,b) ((a)>(b) ? (a) : (b))
#define MIN2(a,b) ((a)<(b) ? (a) : (b))
#define MIX(a,b,c) ((a) + ((b)-(a))*(c))

static int dxdy[][2] = {
		{-1,-2},{0,-2},	{1,-2},	{-2,-1},{-1,-1},
		{0,-1},	{1,-1},	{2,-1},	{-2,0},	{-1,0},
		{1,0},	{2,0},	{-2,1},	{-1,1},	{0,1},
		{1,1},	{2,1},	{-1,2},	{0,2},	{1,2},	{0,0}
	};

static void* generate_binpat(uint16_t side)
{
	int radius = 3;
	uint8_t (*result)[side] = (uint8_t(*)[side])sNewArray(uint8_t, side*side);
	int ddc = sizeof(dxdy)/sizeof(dxdy[0]);
	for (int a=0; a<side*side; a++) {
		int x0=rand()%side;
		int y0=rand()%side;
		int skip = 0;
		if (result[y0][x0]) {
			continue;
		}
		for (int i=0; i<ddc; i++) {
			int x = (x0+dxdy[i][0])%side;
			int y = (y0+dxdy[i][1])%side;
			if (result[y][x]) {
				skip = 1;
				break;
			}
		}
		if (!skip) {
			result[y0][x0] = 255;
		}
	}

	return (void*)result;
}

static void find_cluster_in_binpat(uint8_t* data, int res, int radius, int* crd_x, int* crd_y)
{
	uint8_t (*binpat)[res] = (uint8_t(*)[res])data;	int tvx, tvy;
	int mx=0;
	for (int y0=0; y0<res; y0++) {
		for (int x0=0; x0<res; x0++) {
			if (!binpat[y0][x0]) continue;
			int clust = 0;
			for (int i=0; i<sizeof(dxdy)/sizeof(dxdy[0]); i++) {
				int x=(x0+dxdy[i][1])%res;
				int y=(y0+dxdy[i][0])%res;
				clust += binpat[y][x]!=0;
			}
			if (clust>mx) {
				mx = clust;
				*crd_x = x0;
				*crd_y = y0;
			}
		}
	}
}

static void find_void_in_binpat(uint8_t* data, int res, int radius, int* crd_x, int* crd_y)
{
	uint8_t (*binpat)[res] = (uint8_t(*)[res])data;	int tvx, tvy;
	int mx=0;
	for (int y0=0; y0<res; y0++) {
		for (int x0=0; x0<res; x0++) {
			if (binpat[y0][x0]) continue;
			int _void = 0;
			for (int i=0; i<sizeof(dxdy)/sizeof(dxdy[0]); i++) {
				int x=(x0+dxdy[i][1])%res;
				int y=(y0+dxdy[i][0])%res;
				_void += binpat[y][x]==0;
			}
			if (_void>mx) {
				mx = _void;
				*crd_x = x0;
				*crd_y = y0;
			}
		}
	}
}

sTextureID sTextureGenerateWhiteNoise(int seed, int w, int h)
{
	srand(seed);
	uint8_t *data = sNewArray(uint8_t, w*h*4);
	for (int i=0; i<w*h*4; i++)
	{
		data[i] = rand()&0xFF;
	}
	sTextureID noise = sTextureCreate2D("WhiteNoise", w, h, RGBA8I, 1, 0, data);
	sFree(data);
	return noise;
}

sTextureID sTextureGenerateBlueNoise(int w, int h)
{
	sShaderID filter = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/bng.glsl");
	sTextureID bufferA = sTextureCreate2D("BNA", w, h, RGB8I, 0, 0, 0);
	sTextureID bufferB = sTextureCreate2D("BNB", w, h, RGB8I, 0, 0, 0);
	sFrameBuffer fb = sFrameBufferCreate(w, h, 0);
	sMeshID screen = sMeshCreateScreenPlane();
    sFrameBufferAddRenderTarget(&fb, bufferA);
	sFrameBufferAddRenderTarget(&fb, bufferB);
	sShaderBind(filter);
	sTextureID source;
	sTextureID result;
	for (int i=0;i<3600;i++) {
		source = i&1 ? bufferA : bufferB;
		result = i&1 ? bufferB : bufferA;
		int target = i&1 ? 2 : 1;
		sFrameBufferBind(&fb, target);
		sShaderBindTexture(filter, "channel", source);
		sShaderBindUniformFloat(filter, "width", w);
		sShaderBindUniformFloat(filter, "height", h);
		sShaderBindUniformInt(filter, "iFrame", i);
		glc(sMeshDraw(screen));
	}
	sTextureDelete(source);
	sFrameBufferDelete(&fb);
	sTextureSetTiling(result, sTextureRepeat);
	sMeshDeleteBuffers(screen);
	return result;
}

void sTextureGenerateMipMaps(sTextureID texture)
{
	glc(glBindTexture(texture->type, texture->ID));
	glc(glGenerateMipmap(texture->type));
	//glc(glTexParameteri(texture->type, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
	//glc(glTexParameteri(texture->type, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
}

#ifdef STB_IMAGE_WRITE_IMPLEMENTATION
void sTextureSave(sTextureID texture, const char* fname)
{
	//if (texture->type!=GL_TEXTURE_2D) return;
	int32_t square = (int32_t)texture->width * (int32_t)texture->height;
	int32_t out_cmps = 4;
	switch (texture->color_format)
	{
		case SHADOW16I 	: out_cmps = 1; break;
		case SHADOW32I 	: out_cmps = 1; break;
		case SHADOW32F 	: out_cmps = 1; break;
		case RED8I 		: out_cmps = 1; break;
		case RED16I 	: out_cmps = 1; break;
		case RED32I 	: out_cmps = 1; break;
		case RED16F 	: out_cmps = 1; break;
		case RED32F 	: out_cmps = 1; break;
		case RG8I 		: out_cmps = 3; break;
		case RG16I 		: out_cmps = 3; break;
		case RG32I 		: out_cmps = 3; break;
		case RG16F 		: out_cmps = 3; break;
		case RG32F 		: out_cmps = 3; break;
		case RGB8I 		: out_cmps = 3; break;
		case RGB16I 	: out_cmps = 3; break;
		case RGB32I 	: out_cmps = 3; break;
		case RGB16F 	: out_cmps = 3; break;
		case RGB32F 	: out_cmps = 3; break;
		case RGBA8I 	: out_cmps = 4; break;
		case RGBA16I 	: out_cmps = 4; break;
		case RGBA32I 	: out_cmps = 4; break;
		case RGBA16F 	: out_cmps = 4; break;
		case RGBA32F 	: out_cmps = 4; break;
		default : out_cmps = 4; break;
	}
	//out_cmps = 3;
	void* out_data = sNewArray(uint8_t, square*out_cmps);
	glActiveTexture(GL_TEXTURE0);
	if (texture->parent) {
		glBindTexture(texture->parent->type, texture->ID);
	} else {
		glBindTexture(texture->type, texture->ID);
	}
	puts("glGetTexImage");
	//if (texture->type==GL_TEXTURE_2D)
	{
		switch (out_cmps) {
			case 1 : glc(glGetTexImage(texture->type, 0, GL_RED,  GL_UNSIGNED_BYTE, (void*)out_data)); break;
			case 2 : glc(glGetTexImage(texture->type, 0, GL_RG,   GL_UNSIGNED_BYTE, (void*)out_data)); break;
			case 3 : glc(glGetTexImage(texture->type, 0, GL_RGB,  GL_UNSIGNED_BYTE, (void*)out_data)); break;
			case 4 : glc(glGetTexImage(texture->type, 0, GL_RGBA, GL_UNSIGNED_BYTE, (void*)out_data)); break;
		} 
	} /*else if (texture->parent) {
		switch (out_cmps) {
			case 1 : glc(glGetTextureImage(texture->parent, texture->type - GL_TEXTURE_CUBE_MAP_POSITIVE_X, GL_RED,  GL_UNSIGNED_BYTE, (void*)out_data)); break;
			case 2 : glc(glGetTextureImage(texture->parent, texture->type - GL_TEXTURE_CUBE_MAP_POSITIVE_X, GL_RG,   GL_UNSIGNED_BYTE, (void*)out_data)); break;
			case 3 : glc(glGetTextureImage(texture->parent, texture->type - GL_TEXTURE_CUBE_MAP_POSITIVE_X, GL_RGB,  GL_UNSIGNED_BYTE, (void*)out_data)); break;
			case 4 : glc(glGetTextureImage(texture->parent, texture->type - GL_TEXTURE_CUBE_MAP_POSITIVE_X, GL_RGBA, GL_UNSIGNED_BYTE, (void*)out_data)); break;
		}
	}*/
	printf("Saving %dx%d %d\n", (int32_t)texture->width, (int32_t)texture->height, out_cmps);
	puts("end of glGetTexImage");
	if (fname) {
		char* ext = &(fname[strlen(fname)-4]);
		
		if (ext[0]=='.') {
			if (tolower(ext[1])=='p' && tolower(ext[2])=='n' && tolower(ext[3])=='g')
			{
				stbi_write_png(fname, texture->width, texture->height, out_cmps, (const void*)out_data, texture->width*out_cmps);
			}
			else if (tolower(ext[1])=='j' && tolower(ext[2])=='p' && tolower(ext[3])=='g')
			{
				stbi_write_jpg(fname, texture->width, texture->height, out_cmps, (const void*)out_data, 100);
			}
			else if (tolower(ext[1])=='t' && tolower(ext[2])=='g' && tolower(ext[3])=='a')
			{
				stbi_write_tga(fname, texture->width, texture->height, out_cmps, (const void*)out_data);
			}
			else if (tolower(ext[1])=='b' && tolower(ext[2])=='m' && tolower(ext[3])=='p')
			{
				stbi_write_bmp(fname, texture->width, texture->height, out_cmps, (const void*)out_data);
			}
			else
			{
				stbi_write_png(fname, texture->width, texture->height, out_cmps, (const void*)out_data, texture->width*out_cmps);
			}
		}
	} else {
		char fn[strlen(texture->name)+5];
		sprintf(fn, "%s.png", texture->name);
		//FILE* fp = fopen(fn, "wb");
		//fwrite((void*)(int[2]){texture->width, texture->height}, sizeof(int), 2, fp);
		//fwrite(out_data, square, out_cmps, fp);
		//fclose(fp);
		stbi_write_png(fn, texture->width, texture->height, out_cmps, (const void*)out_data, texture->width*out_cmps);
	}
	sDelete(out_data);
}
#endif

#ifdef STB_IMAGE_IMPLEMENTATION
sTextureID sTextureLoad(const char* fname, const char* tname)
{
	sTextureID texture;
	if (!fname) return 0;
	int cmps, width, height;
	stbi_uc *data = stbi_load(fname, &width, &height, &cmps, 0);
	printf("Loaded %s %dx%dx%d\n", fname, width, height, cmps);
	
	switch (cmps)
	{
	case 1 : texture = sTextureCreate2D(tname ? tname : fname, width, height, GRAY8I, 0, 0, (void*)data); break;
	case 3 : texture = sTextureCreate2D(tname ? tname : fname, width, height, RGB8I,  0, 0, (void*)data); break;
	case 4 : texture = sTextureCreate2D(tname ? tname : fname, width, height, RGBA8I, 0, 0, (void*)data); break;
	}
	stbi_image_free((void*)data);
	return texture;
}
#endif

size_t sTextureGetQuantity(void)
{
    return sListGetSize(textures);
}

void sTexturePrintUsers(sTextureID tex)
{
    size_t users = sListGetSize(tex->material_users);
    printf("sTexturePrintUsers(sTextureID->name \"%s\")\n", tex->name);
    printf("%lu material users\n", users);
    for (size_t i=0; i<users; i++)
    {
        puts(tex->material_users[i]->name);
    }
}

void sTextureClear(void)
{
	size_t tex_count = sListGetSize(textures);
	sTextureID* texs = sNewArray(sTextureID, tex_count);
	memcpy(texs, textures, sSizeof(textures));
	for (size_t i=0; i<tex_count; i++)
	{
		if (texs[i]->parent) continue;
		if (!texs[i]->fake_user && !texs[i]->material_users && !texs[i]->framebuffer_users)
		{
			printf("Удаляется sTexture(%s)\n", texs[i]->name);
			sTextureDelete(texs[i]);
		} else {
            printf("sTexture(%s) имеет пользователей:\n", texs[i]->name);
			if (texs[i]->fake_user) {
				puts("  фейковый");
			}
            for (size_t m=0; m<sListGetSize(texs[i]->framebuffer_users); m++) {
                printf("  sFrameBuffer(%p)\n", texs[i]->framebuffer_users);
            }
            for (size_t m=0; m<sListGetSize(texs[i]->material_users); m++) {
                printf("  sMaterial(%s)\n", texs[i]->material_users[m]->name);
            }
        }
    	puts("");
	}
	sDelete(texs);
}


#ifdef __cplusplus
}
#endif
