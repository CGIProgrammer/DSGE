
/*
 * types.h
 *
 *  Created on: 9 июль 2020 г.
 *      Author: Ivan G
 */

#include <stdint.h>

#ifndef TYPES_H
#define TYPES_H

#define FORWARD
#define GLAD

#ifdef GLAD
#include "glad/glad.h"
#endif
#ifdef GLEW
#include <GL/glew.h>
#endif

typedef void(*procedure)(void);

#ifndef GL_GINORE_ERRORS
#define glc(func) {func;GLenum err=glGetError(); if (err) {printf("GLerror:%s:%d : %d\n",__FILE__,__LINE__,err);/*((procedure)0)();*/}}
#else
#define glc(func) func
#endif

#ifndef __cplusplus
	#ifndef bool
		typedef _Bool bool;
	#endif
	#ifndef null
		#define null ((void*)0)
	#endif
	#ifndef true
		#define true 1
	#endif
	#ifndef false
		#define false 0
	#endif
#endif

typedef enum {
    gAlbedoIndex = 0,
    gSpaceIndex,
    gMasksIndex,
    gAmbientIndex,
    gOutputAIndex,
    gOutputBIndex,
    gOutputDiffuseIndex,
    gOutputSpecularIndex,
	gRenderAccumulator1,
	gRenderAccumulator2,
	gVectors
} RenderTargetBit;

#define gAlbedoBit (1<<gAlbedoIndex)
#define gSpaceBit (1<<gSpaceIndex)
#define gMasksBit (1<<gMasksIndex)
#define gAmbientBit (1<<gAmbientIndex)
#define gOutputABit (1<<gOutputAIndex)
#define gOutputBBit (1<<gOutputBIndex)
#define gOutputDiffuseBit (1<<gOutputDiffuseIndex)
#define gOutputSpecularBit (1<<gOutputSpecularIndex)
#define gRenderAccumulator1Bit (1<<gRenderAccumulator1)
#define gRenderAccumulator2Bit (1<<gRenderAccumulator2)
#define gVectorsBit (1<<gVectors)

typedef struct
{
	float r,g,b,a;
} sColor;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
	float u2,v2;
	uint32_t w1,w2,w3;
} sVertex;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
	uint32_t w1,w2,w3;
} bVertex;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
} smVertex;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
	uint32_t w1,w2,w3;
	uint32_t u2,v2;
} smbVertex;

typedef struct
{
	sVertex vert[3];
} sTriangle;

typedef enum sShaderType {
	sShaderBase,
	sShaderSkeleton,
	sShaderBaseShadow,
	sShaderSkeletonShadow
} sShaderType;

typedef uint32_t index_t;

typedef struct sTexture* sTextureID;
typedef struct sShader* sShaderID;
typedef struct sScene* sSceneID;
typedef struct sMesh* sMeshID;
typedef struct sMaterial* sMaterialID;
typedef struct sGameObject* sGameObjectID;
typedef void (*sGameObjectCallback)(sGameObjectID);
typedef struct sFrameBuffer* sFrameBufferID;
typedef struct sCameraComponent* sCameraComponentID;
typedef struct sLightComponent* sLightComponentID;
typedef void (*sCameraRenderPipelineCallback)(sCameraComponentID, sGameObjectID*, sGameObjectID*, sTextureID);
typedef void (*sLightRenderPipelineCallback)(sLightComponentID, sGameObjectID*);

typedef struct sFrameBuffer
{
    uint16_t width, height;
    GLuint framebuffer_id;
    GLuint renderbuffer_id;
    sTextureID color_render_targets[16];
    sTextureID depth_render_target;
} sFrameBuffer;

#endif
