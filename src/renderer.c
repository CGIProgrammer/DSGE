/*
 * renderer.c
 *
 *  Created on: 4 авг. 2018 г.
 *      Author: ivan
 */
#include "engine.h"

#define VOXEL_MAP_SIZE 30.0
#define VOXEL_SIZE VOXEL_MAP_SIZE/256.0

#define FONT_TABLE_WIDTH  16
#define FONT_TABLE_HEIGHT 16

char buff[1024];
uint32_t activeShader;
_Bool _renderDeferred = 1;
_Bool _renderVectors = 0;
_Bool _renderHDR = 0;
_Bool _renderRayTracing = 1;
_Bool _renderBloom = 0;
_Bool _renderApplyChanges = 0;
_Bool _renderReflections = 1;
_Bool _VXGI = 0;

sShader base_shader,skin_shader,shader_shadowmap,skin_shader_shadowmap;
sShader vectorsShader,skin_vectorsShader;
sShader voxelShader,skin_voxelShader;
sShader UI_planeShader,UI_charShader;
sShader glassShader,skin_glassShader;
sShader ssgi_shader, ssgi_denoise_shader, bloom_shader, deferred_lighting_shader, motion_blur_shader;
sShader ssr_shader;

sTexture noise_texture;

sVertex render_plane[] =
{{-1.0,-1.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 0.0},
{-1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 1.0},
{ 1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 1.0},
{ 1.0,-1.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 0.0}};
index_t render_plane_ind[] = {0,3,2,2,1,0};

sVertex ui_plane[] =
{{ 0.0, 0.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 0.0},
 { 0.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 1.0},
 { 1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 1.0},
 { 1.0, 0.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 0.0}};
index_t ui_plane_ind[] = {0,3,2,2,1,0};

enum
{
	GRAY8I = 0,
	GRAY16I,
	GRAY32I,
	GRAY16F,
	GRAY32F,

	RED8I = 0,
	RED16I,
	RED32I,
	RED16F,
	RED32F,

	RG8I,
	RG16I,
	RG32I,
	RG16F,
	RG32F,

	RGB8I,
	RGB16I,
	RGB32I,
	RGB16F,
	RGB32F,

	RGBA8I,
	RGBA16I,
	RGBA32I,
	RGBA16F,
	RGBA32F,

	SHADOW16I,
	SHADOW32I,
	SHADOW32F
};
static int formats[][3] = {\
			{GL_RED,	GL_R8,		GL_UNSIGNED_BYTE},
			{GL_RED,	GL_R16,		GL_UNSIGNED_SHORT},
			{GL_RED,	GL_R32UI,	GL_UNSIGNED_INT},
			{GL_RED,	GL_R16F,	GL_HALF_FLOAT},
			{GL_RED,	GL_R32F,	GL_FLOAT},
			{GL_RG,		GL_RG8,		GL_UNSIGNED_BYTE},
			{GL_RG,		GL_RG16,	GL_UNSIGNED_SHORT},
			{GL_RG,		GL_RG32UI,	GL_UNSIGNED_INT},
			{GL_RG,		GL_RG16F,	GL_HALF_FLOAT},
			{GL_RG,		GL_RG32F,	GL_FLOAT},
			{GL_RGB,	GL_RGB8,	GL_UNSIGNED_BYTE},
			{GL_RGB,	GL_RGB16,	GL_UNSIGNED_SHORT},
			{GL_RGB,	GL_RGB32UI,	GL_UNSIGNED_INT},
			{GL_RGB,	GL_RGB16F, 	GL_HALF_FLOAT},
			{GL_RGB,	GL_RGB32F, 	GL_FLOAT},
			{GL_RGBA,	GL_RGBA8,  	GL_UNSIGNED_BYTE},
			{GL_RGBA,	GL_RGBA16, 	GL_UNSIGNED_SHORT},
			{GL_RGBA, 	GL_RGBA32UI,GL_UNSIGNED_INT},
			{GL_RGBA,	GL_RGBA16F, GL_HALF_FLOAT},
			{GL_RGBA,	GL_RGBA32F, GL_FLOAT},
			{GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT16, GL_UNSIGNED_SHORT},
			{GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT32, GL_UNSIGNED_INT},
			{GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT32F, GL_FLOAT}
};

static GLuint GenRenderTexture(int width, int height, int pix_format, int mipmaps, int filtering)
{

	GLuint result;
	GLint internal_format = formats[pix_format][1];
	GLint format = formats[pix_format][0];
	GLint type = formats[pix_format][2];
	glGenTextures(1, &result);
	glBindTexture(GL_TEXTURE_2D, result);
	glTexImage2D(GL_TEXTURE_2D, 0,internal_format, width, height, 0,format, type, 0);
	if (mipmaps)
	{
		glGenerateMipmap(GL_TEXTURE_2D);
		glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR);
	}
	else
	{
		glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filtering ? GL_LINEAR : GL_NEAREST);
	}
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filtering ? GL_LINEAR : GL_NEAREST);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
	return result;
}

/*static void BindTexturesToRenderBuffer(GLuint fb, int count, int* targets, int depth_target)
{
	GLint attachments[32];
	for (uint32_t i=0; i<count; i++)
	{
		glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0 + i, GL_TEXTURE_2D, targets[i], 0);
		attachments[i] = GL_COLOR_ATTACHMENT0 + i;
	}
	if (depth_target)
	{
		glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, depth_target);
	}
	glDrawBuffers(count, (const GLenum*)attachments);
}*/

sScene* sObjectGetScene(void* object)
{
    return ((sObject*)object)->scene;
}

void sObjectSetMaterialFriction(sObject* obj,float friction)
{
    obj->mesh->material->friction = friction;
}

void sMaterialUniformLA(GLuint mat,char* name,laType* data)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    switch (data->type)
    {
        case VECTOR : {
        	glUniform3f(uniform,data->a[0],data->a[1],data->a[2]);
        	break;
        }
        case MATRIX : {
            glUniformMatrix4fv(uniform,1,GL_FALSE,data->a);
            break;
        }
        case 12 :
        {
        	glUniformMatrix3x4fv(uniform,1,GL_FALSE,data->a);
            break;
        }
    }
}

void sMaterialTexture(GLuint id,char* name,uint32_t ID,GLint index)
{
    GLint uniform;
    uniform = glGetUniformLocation(id,name);
    if (uniform==-1)
    {
		//printf("Uniform %s not found\n",name);
		return;
	}
    glEnable(GL_TEXTURE_2D);
    glActiveTexture(GL_TEXTURE0+index);
    glBindTexture(GL_TEXTURE_2D, ID);
    glUniform1i(uniform, index);

}

void sMaterialTextureCube(GLuint id,char* name,uint32_t ID,GLint index)
{
    GLint uniform;
    uniform = glGetUniformLocation(id,name);
    if (uniform==-1)
    {
		//printf("Uniform %s not found\n",name);
		return;
	}
    glEnable(GL_TEXTURE_CUBE_MAP);
    glActiveTexture(GL_TEXTURE0+index);
    glBindTexture(GL_TEXTURE_CUBE_MAP, ID);
    glUniform1i(uniform, index);

}

void sMaterialTextureArray(GLuint id,char* name,uint32_t ID,GLint index)
{
    GLint uniform;
    uniform = glGetUniformLocation(id,name);
    if (uniform==-1)
    {
		//printf("Uniform %s not found\n",name);
		return;
	}
    glc(glActiveTexture(GL_TEXTURE0+index));
    glc(glBindTexture(GL_TEXTURE_2D_ARRAY, ID));
    glc(glUniform1i(uniform, index));

}

void sMaterialTexture3D(GLuint id,char* name,uint32_t ID,GLint index)
{
    GLint uniform;
    uniform = glGetUniformLocation(id,name);
    if (uniform==-1)
    {
		//printf("Uniform %s not found\n",name);
		return;
	}
    glc(glActiveTexture(GL_TEXTURE0+index));
    glc(glBindTexture(GL_TEXTURE_3D, ID));
    glc(glUniform1i(uniform, index));

}

void sMaterialTextureID(GLuint id,char* name,sTexture* ID,GLint index)
{
    GLint uniform;
    uniform = glGetUniformLocation(id,name);
    if (uniform==-1) return;
    /*{
                    printf("Uniform %s not found\n",name);
                    exit(-1);
    }*/
    //printf("Texture %s ID %d\n",name,id);
    glEnable(ID->type);
    glActiveTexture(GL_TEXTURE0+index);
    glBindTexture(ID->type, ID->ID);
    glUniform1i(uniform, index);

}

void S_MATERIAL_TextureMS(GLuint id,char* name,GLint ID,GLint index)
{
    GLint uniform;
    uniform = glGetUniformLocation(id,name);
    if (uniform==-1) return;
    /*{
     *                printf("Uniform %s not found\n",name);
     *                exit(-1);
}*/
    glEnable(GL_TEXTURE_2D_MULTISAMPLE);
    glActiveTexture(GL_TEXTURE0+index);
    glBindTexture(GL_TEXTURE_2D_MULTISAMPLE, ID);
    glUniform1i(uniform, index);

}

void sMaterialUniformfv(GLuint mat,char* name,void* data,GLuint count)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    switch (count)
    {
        case 2 : glUniform2fv(uniform,1,data);break;
        case 3 : glUniform3fv(uniform,1,data);break;
        case 4 : glUniform4fv(uniform,1,data);break;
        case 9 : glUniformMatrix3fv(uniform,1,GL_FALSE,data);break;
        case 12 : glUniformMatrix3x4fv(uniform,1,GL_FALSE,data);break;
        case 16 : glUniformMatrix4fv(uniform,1,GL_FALSE,data);break;
        default : glUniform1fv(uniform,count,data);break;
    }
}

void sMaterialUniformf(GLuint mat,char* name,float data)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    glUniform1f(uniform,data);
}

void S_MATERIAL_Uniform2f(GLuint mat,char* name,float data,float data2)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    glUniform2f(uniform,data,data2);
}

void S_MATERIAL_Uniform3f(GLuint mat,char* name,float data1,float data2,float data3)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    glUniform3f(uniform,data1,data2,data3);
}

void sMaterialUniformiv(GLuint mat,char* name,void* data,GLuint count)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    switch (count)
    {
        case 1 : glUniform1iv(uniform,1,data);break;
        case 2 : glUniform2iv(uniform,1,data);break;
        case 3 : glUniform3iv(uniform,1,data);break;
        case 4 : glUniform4iv(uniform,1,data);break;
        default : glUniform1fv(uniform,count,data);break;
    }
}

void sMaterialStructLA(GLuint mat,char* name,char* attr,int index,laType* data)
{
    GLint uniform;
    sprintf(buff,"%s[%d].%s",name,index,attr);
    uniform = glGetUniformLocation(mat,buff);
    if (uniform==-1) return;
    switch (data->type)
    {
        case VECTOR : {
        	glUniform3f(uniform,data->a[0],data->a[1],data->a[2]);
        	break;
        }
        case MATRIX : {
            glUniformMatrix4fv(uniform,1,GL_FALSE,data->a);
            break;
        }
        case 12 : {
            glUniformMatrix3x4fv(uniform,1,GL_FALSE,data->a);
            break;
        }
    }
}

void sMaterialStructfv(GLuint mat,char* name,char* attr,int index,void* data,uint32_t count)
{
    GLint uniform;
    sprintf(buff,"%s[%d].%s",name,index,attr);
    uniform = glGetUniformLocation(mat,buff);
    if (uniform==-1) return;
    switch (count)
    {
        case 2 : glUniform2fv(uniform,1,data);break;
        case 3 : glUniform3fv(uniform,1,data);break;
        case 4 : glUniform4fv(uniform,1,data);break;
        case 9 : glUniformMatrix3fv(uniform,1,GL_FALSE,data);break;
        case 12 : glUniformMatrix3x4fv(uniform,1,GL_FALSE,data);break;
        case 16 : glUniformMatrix4fv(uniform,1,GL_FALSE,data);break;
        default : glUniform1fv(uniform,count,data);break;
    }
}

void sMaterialStructiv(GLuint mat,char* name,char* attr,int index,void* data,uint32_t count)
{
    GLint uniform;
    sprintf(buff,"%s[%d].%s",name,index,attr);
    uniform = glGetUniformLocation(mat,buff);
    if (uniform==-1) return;
    switch (count)
    {
        case 2 : glUniform2iv(uniform,1,data);break;
        case 3 : glUniform3iv(uniform,1,data);break;
        case 4 : glUniform4iv(uniform,1,data);break;
        default : glUniform1iv(uniform,count,data);break;
    }
}

void sMaterialUniformi(GLuint mat,char* name,long num)
{
    GLint uniform;
    uniform = glGetUniformLocation(mat,name);
    if (uniform==-1) return;
    glUniform1i(uniform,num);
}

void sRenderSetReflections(int val)
{
	printf("Reflections %s\n", val ? "on" : "off");
	_renderReflections = val;
}

int sRenderGetReflections(void)
{
	return _renderReflections;
}

void sRenderSetSSGI(int val)
{
	printf("SSGI %s\n", val ? "on" : "off");
	_renderRayTracing = val;
}

int sRenderGetSSGI(void)
{
	return _renderRayTracing;
}

void sRenderSetMotionBlur(int val)
{
	printf("MotionBlur %s\n", val ? "on" : "off");
	_renderVectors = val;
}

int sRenderGetMotionBlur(void)
{
	return _renderVectors;
}

void sRenderSetHDR(int val)
{
	printf("HDR %s\n", val ? "on" : "off");
	_renderHDR = val;
}

int sRenderGetHDR(void)
{
	return _renderHDR;
}

void sRenderSetBloom(int val)
{
	printf("Bloom %s\n", val ? "on" : "off");
	_renderBloom = val;
}

int sRenderGetBloom(void)
{
	return _renderBloom;
}

void sCameraTakeScreenshot(sCamera* camera, char* file_name)
{
	uint32_t dims[4];
	char full_path[512];
	FILE* fp;
	
	glBindFramebuffer(GL_FRAMEBUFFER, 0);
	glGetIntegerv(GL_VIEWPORT, (GLint*)dims);
	
	uint32_t width = dims[2], height = dims[3];
	uint8_t image_data[width][3];
	
	uint8_t bmp_header[54];
	bmp_header[1] = 0x4D;
	bmp_header[0] = 0x42;
	*(uint32_t*)(bmp_header + 2) = 54 + dims[2]*dims[3]*3;
	*(uint16_t*)(bmp_header + 6) = 0;
	*(uint16_t*)(bmp_header + 8) = 0;
	*(uint32_t*)(bmp_header +10) = 54;
	*(uint32_t*)(bmp_header +14) = 40;
	*(uint32_t*)(bmp_header +18) = dims[2];
	*(uint32_t*)(bmp_header +22) = dims[3];
	*(uint16_t*)(bmp_header +26) = 1;
	*(uint16_t*)(bmp_header +28) = 24;
	*(uint32_t*)(bmp_header +30) = 0;
	*(uint32_t*)(bmp_header +34) = dims[2]*dims[3]*3;
	*(uint32_t*)(bmp_header +38) = 2000;
	*(uint32_t*)(bmp_header +42) = 2000;
	*(uint32_t*)(bmp_header +46) = 0;
	*(uint32_t*)(bmp_header +50) = 0;
	
	sprintf(full_path, "screenshots/%s.bmp", file_name);
	for (char* space=strchr(full_path, ':');space;space=strchr(full_path, ':'))
	{
		*space = '-';
	}
	for (char* space=strchr(full_path, ' ');space;space=strchr(full_path, ' '))
	{
		*space = '_';
	}
	for (char* space=strchr(full_path, '\n');space;space=strchr(full_path, '\n'))
	{
		strcpy(space, space+1);
	}
	if (!(fp = fopen(full_path, "wb")))
	{
		fprintf(stderr, "File to save %s\n", full_path);
		return;
	}
	fwrite(bmp_header, 54, 1, fp);
	for (uint32_t i=0;i<height;i++)
	{
		glReadPixels(0, i, width, 1, GL_BGR, GL_UNSIGNED_BYTE, (void*)&image_data);
		fwrite((void*)image_data, width, 3, fp);
	}
	fclose(fp);
	fprintf(LOGOUT, "Saved %s\n",full_path);
}

void sCameraInitShadowFB(sCamera* camera)
{
    camera->render_plane.indices = render_plane_ind;
    camera->render_plane.vertices = render_plane;
    camera->render_plane.vert_count = sizeof(render_plane)/sizeof(render_plane[0]);
    camera->render_plane.ind_count = sizeof(render_plane_ind)/sizeof(render_plane_ind[0]);

    glc(glGenFramebuffers(1,&camera->render_fb));
    glc(glBindFramebuffer(GL_FRAMEBUFFER, camera->render_fb));
    camera->render_texture = GenRenderTexture(camera->width, camera->height, SHADOW16I, 0, 0);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, camera->render_texture, 0);
}

void sCameraDestroyFB(sCamera* camera)
{
	glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
	glc(glBindTexture(GL_TEXTURE_2D, 0));
	if (camera->render_ambient) glc(glDeleteTextures(1, &camera->render_ambient));	//1
	if (camera->render_normal_glass) glc(glDeleteTextures(1, &camera->render_normal_glass));	//2
	if (camera->render_normal) glc(glDeleteTextures(1, &camera->render_normal));	//3
	if (camera->render_result) glc(glDeleteTextures(1, &camera->render_result));	//4
	if (camera->render_specular) glc(glDeleteTextures(1, &camera->render_specular));//5
	if (camera->render_texture) glc(glDeleteTextures(1, &camera->render_texture));	//6
	if (camera->render_texture1) glc(glDeleteTextures(1, &camera->render_texture1));//7
	if (camera->render_texture2) glc(glDeleteTextures(1, &camera->render_texture2));//8
	if (camera->render_vectors) glc(glDeleteTextures(1, &camera->render_vectors));	//9
	if (camera->render_depth) glc(glDeleteRenderbuffers(1, &camera->render_depth));	//10
	if (camera->render_fb) glc(glDeleteFramebuffers(1, &camera->render_fb));		//11

	sMeshDelete(&camera->render_plane);
}

void sRenderSwapPPShaders(void)
{
	_renderApplyChanges = 1;
}

void sRenderLoadShaders(void)
{

	glc(sLoadVertexFromFile(&base_shader,"data/shaders/base_shaders/base_vert.glsl"));
	if (_renderDeferred)
	{
		glc(sLoadFragmentFromFile(&base_shader,"data/shaders/base_shaders/base_frag.glsl"));
	}
	else
	{
		glc(sLoadFragmentFromFile(&base_shader,"data/shaders/base_shaders/base_frag_lighting.glsl"));
	}
	glc(sShaderMake(&base_shader));

	skin_shader.fragment = base_shader.fragment;
	glc(sLoadVertexFromFile(&skin_shader,"data/shaders/base_shaders/skin_vert.glsl"));
	glc(sShaderMake(&skin_shader));

	glc(sLoadVertexFromFile(&shader_shadowmap,"data/shaders/base_shaders/base_vert.glsl"));
	glc(sLoadFragmentFromFile(&shader_shadowmap,"data/shaders/base_shaders/shadow_map.glsl"));
	glc(sShaderMake(&shader_shadowmap));
	glc(sLoadVertexFromFile(&skin_shader_shadowmap,"data/shaders/base_shaders/skin_vert.glsl"));
	glc(sLoadFragmentFromFile(&skin_shader_shadowmap,"data/shaders/base_shaders/shadow_map.glsl"));
	glc(sShaderMake(&skin_shader_shadowmap));

	if (_renderVectors)
	{
		sLoadVertexFromFile(&vectorsShader,"data/shaders/motion_blur/vectors_base_vert.glsl");
		sLoadFragmentFromFile(&vectorsShader,"data/shaders/motion_blur/vectors_base_frag.glsl");
		sShaderMake(&vectorsShader);

		sLoadVertexFromFile(&skin_vectorsShader,"data/shaders/motion_blur/vectors_skin_vert.glsl");
		sLoadFragmentFromFile(&skin_vectorsShader,"data/shaders/motion_blur/vectors_base_frag.glsl");
		sShaderMake(&skin_vectorsShader);
	}

	uint32_t plane_shader;
	//sLoadVertexFromFile(&UI_planeShader,"data/shaders/UI/element_plane.glsl");
	//sLoadFragmentFromFile(&UI_planeShader,"data/shaders/UI/element.glsl");
	//sShaderMake(&UI_planeShader);

	//sLoadVertexFromFile(&UI_charShader,"data/shaders/UI/text_plane.glsl");
	//sLoadFragmentFromFile(&UI_charShader,"data/shaders/UI/text.glsl");
	//sShaderMake(&UI_charShader);

	sLoadVertexFromFile(&deferred_lighting_shader,"data/shaders/postprocessing/screen_plane.glsl");
	plane_shader = deferred_lighting_shader.vertex;
	if (_renderDeferred)
	{
		sLoadFragmentFromFile(&deferred_lighting_shader,"data/shaders/lighting/dsf.glsl");
		sShaderMake(&deferred_lighting_shader);

		if (_renderReflections)
		{
			sLoadFragmentFromFile(&ssr_shader,"data/shaders/postprocessing/filter_r_rt.glsl");
			ssr_shader.vertex = plane_shader;
			sShaderMake(&ssr_shader);
		}
	}
	else
	{
		sLoadFragmentFromFile(&deferred_lighting_shader,"data/shaders/postprocessing/frpp.glsl");
		sShaderMake(&deferred_lighting_shader);
	}

	if (_renderVectors)
	{
		sLoadFragmentFromFile(&motion_blur_shader,"data/shaders/postprocessing/motion_blur.glsl");
		motion_blur_shader.vertex = plane_shader;
		sShaderMake(&motion_blur_shader);
	}

	if (_renderBloom)
	{
		sLoadFragmentFromFile(&bloom_shader,"data/shaders/postprocessing/bloom.glsl");
		bloom_shader.vertex = plane_shader;
		sShaderMake(&bloom_shader);
	}

	if (_renderRayTracing)
	{
		ssgi_shader.vertex = plane_shader;
		ssgi_denoise_shader.vertex = plane_shader;
		sLoadFragmentFromFile(&ssgi_shader,"data/shaders/raytracing/global_illumination.glsl");
		sLoadFragmentFromFile(&ssgi_denoise_shader,"data/shaders/raytracing/denoise.glsl");
		sShaderMake(&ssgi_shader);
		sShaderMake(&ssgi_denoise_shader);
	}
}

void sRenderDestroyShaders(void)
{
	sShaderDestroy(&base_shader);
	sShaderDestroy(&skin_shader);

	sShaderDestroy(&shader_shadowmap);
	sShaderDestroy(&skin_shader_shadowmap);

	sShaderDestroy(&vectorsShader);
	sShaderDestroy(&skin_vectorsShader);

	sShaderDestroy(&UI_planeShader);
	sShaderDestroy(&UI_charShader);
	sShaderDestroy(&deferred_lighting_shader);
	sShaderDestroy(&ssgi_shader);
	sShaderDestroy(&ssgi_denoise_shader);
	sShaderDestroy(&ssr_shader);
	sShaderDestroy(&motion_blur_shader);
	sShaderDestroy(&bloom_shader);
}


void sCameraInitFB(sCamera* camera)
{
	memset(camera->filters, 0, sizeof(camera->filters));

	camera->width = sEngineGetWidth();
	camera->height = sEngineGetHeight();

    camera->render_plane.indices = render_plane_ind;
    camera->render_plane.vertices = render_plane;
    camera->render_plane.vert_count = sizeof(render_plane)/sizeof(render_plane[0]);
    camera->render_plane.ind_count = sizeof(render_plane_ind)/sizeof(render_plane_ind[0]);
    glc(sMeshMakeBuffers(&camera->render_plane));

    camera->render_result = GenRenderTexture(camera->width, camera->height, _renderHDR ? RGB16F : RGB8I, _renderBloom, _renderBloom);
    camera->render_texture1 = GenRenderTexture(camera->width, camera->height, _renderHDR ? RGB16F : RGB8I, _renderBloom, _renderBloom);
    camera->render_texture2 = GenRenderTexture(camera->width, camera->height, _renderHDR ? RGB16F : RGB8I, _renderBloom, _renderBloom);

    camera->noise = noise_texture.ID;

	if (_renderDeferred)
	{
		camera->render_texture = GenRenderTexture(camera->width, camera->height, RGBA8I, 0, 1);
		camera->render_normal = GenRenderTexture(camera->width, camera->height, RGBA16F, 0, 0);
		camera->render_normal_glass = GenRenderTexture(camera->width, camera->height, RGBA16F, 0, 1);
		camera->render_ambient = GenRenderTexture(camera->width, camera->height, RGB16F, 0, 1);
		camera->render_specular = GenRenderTexture(camera->width, camera->height, RGB8I, 0, 1);
	}
	
    if (_renderVectors)
    {
    	camera->render_vectors = GenRenderTexture(camera->width, camera->height, RG16F, 0, 0);
    }

    //puts("Init frame buffers");
    glc(glGenRenderbuffers(1, &camera->render_depth));
    glc(glBindRenderbuffer(GL_RENDERBUFFER, camera->render_depth));
    glc(glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT32, camera->width, camera->height));
    glc(glBindRenderbuffer(GL_RENDERBUFFER, 0));

    glc(glGenFramebuffers(1,&camera->render_fb));
    glc(glBindFramebuffer(GL_FRAMEBUFFER, camera->render_fb));
	
    glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));

    camera->filters[0] = &deferred_lighting_shader;
	camera->mipmap_layers = 0;

	if (_renderDeferred)
	{
		if (_renderRayTracing)
		{
			sCameraAddPPShader(camera,&ssgi_shader);
			sCameraAddPPShader(camera,&ssgi_denoise_shader);
		}
		if (_renderReflections)
		{
			sCameraAddPPShader(camera,&ssr_shader);
			camera->mipmap_layers |= 1<<(sCameraGetFiltersCount(camera));
		}
		if (_renderVectors)
		{
			sCameraAddPPShader(camera,&motion_blur_shader);
		}
		if (_renderBloom)
		{
			sCameraAddPPShader(camera,&bloom_shader);
			camera->mipmap_layers |= 1<<(sCameraGetFiltersCount(camera));
		}
	}
}

void sCameraResizeFB(sCamera *camera)
{

}

void sCameraAddFilter(sCamera* camera,char* file)
{
	uint8_t filters_count = 0;
	for (filters_count=0;filters_count<8 && camera->filters[filters_count]->program;filters_count++);
	if (filters_count==8)
	{
		fprintf(stderr,"Too many filters attached\n");
		return;
	}
	else
	{
		//fprintf(LOGOUT, "Filter #%d\n",filters_count);
	}
	char shader_path[256];
	sprintf(shader_path,"data/shaders/%s",file);

	camera->filters[filters_count]->vertex = camera->filters[0]->vertex;
	sLoadFragmentFromFile(camera->filters[filters_count],shader_path);
	sShaderMake(camera->filters[filters_count]);
}

void sCameraAddPPShader(sCamera* camera,sShader* shader)
{
	uint8_t filters_count = 0;
	for (filters_count=0;filters_count<8 && camera->filters[filters_count];filters_count++);
	if (filters_count==8)
	{
		fprintf(stderr,"Too many filters attached\n");
		return;
	}
	else
	{
		//fprintf(LOGOUT, "Filter #%d\n",filters_count);
	}

	camera->filters[filters_count] = shader;
}

uint8_t sCameraGetFiltersCount(sCamera* camera)
{
	for (uint8_t i = 0; i < 8; i++)
	{
		if (!camera->filters[i+1])
		{
			return i;
		}
	}
	return 8;
}

void sCameraBindVectorsFB(sCamera* camera)
{
	glc(glBindFramebuffer(GL_FRAMEBUFFER,camera->render_fb));
	glc(glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_vectors, 0));

    GLuint attachment = GL_COLOR_ATTACHMENT0;
    glc(glDrawBuffers(1, &attachment));

    glc(glClearColor(0.0,0.0,0.0,1.0));
    glc(glClear(GL_DEPTH_BUFFER_BIT|GL_COLOR_BUFFER_BIT));
	glc(glDisable(GL_BLEND));
	glc(glDisable(GL_ALPHA_TEST));
	glc(glEnable(GL_CULL_FACE));
	glc(glEnable(GL_DEPTH_TEST));
	glc(glDepthFunc(GL_LEQUAL));
	glc(glViewport(0,0,camera->width,camera->height));
	camera->projection = Perspective(camera->width,camera->height,camera->zFar,camera->zNear,camera->FOV);
	camera->viewProjection = Mul(camera->projection,Inverted(sCameraGetTransform(camera)));

    useProgram(128);
}

void sCameraBindFB(sCamera* camera, int transparency)
{
    glc(glBindFramebuffer(GL_FRAMEBUFFER,camera->render_fb));
    glc(glDisable(GL_BLEND));
    glc(glDisable(GL_ALPHA_TEST));
    glc(glEnable(GL_CULL_FACE));
    glc(glEnable(GL_DEPTH_TEST));
    glc(glDepthFunc(GL_LEQUAL));
    glc(glViewport(0,0,camera->width,camera->height));

    if (camera->name[0]=='c')
    {
        
		GLuint attachments[] = { GL_COLOR_ATTACHMENT0, GL_COLOR_ATTACHMENT1, GL_COLOR_ATTACHMENT2, GL_COLOR_ATTACHMENT3, GL_COLOR_ATTACHMENT4, GL_COLOR_ATTACHMENT5 };
		if (_renderDeferred)
		{
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT4, GL_TEXTURE_2D, camera->render_normal_glass, 0);
			if (transparency)
			{
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_texture, 0);
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT1, GL_TEXTURE_2D, camera->render_normal, 0);
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT2, GL_TEXTURE_2D, camera->render_ambient, 0);
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT3, GL_TEXTURE_2D, camera->render_specular, 0);
			}
			else
			{
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, 0, 0);
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT1, GL_TEXTURE_2D, 0, 0);
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT2, GL_TEXTURE_2D, 0, 0);
				glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT3, GL_TEXTURE_2D, 0, 0);
			}
			glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, camera->render_depth);

			glDrawBuffers(5, attachments);
		}
		else
		{
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_texture1, 0);
			glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, camera->render_depth);
			glDrawBuffers(1, attachments);
		}
    }

	camera->viewProjection = Mul(camera->projection,Inverted(sCameraGetTransform(camera)));
    useProgram(128);
}

void sCameraClearRenderTargets(sCamera* camera)
{
	glc(glBindFramebuffer(GL_FRAMEBUFFER,camera->render_fb));
	glc(glDisable(GL_BLEND));
	glc(glDisable(GL_ALPHA_TEST));
	glc(glEnable(GL_CULL_FACE));
	glc(glEnable(GL_DEPTH_TEST));
	glc(glDepthFunc(GL_LEQUAL));
	glc(glViewport(0,0,camera->width,camera->height));

	if (camera->name[0]=='c')
	{

		GLuint attachments[] = { GL_COLOR_ATTACHMENT0, GL_COLOR_ATTACHMENT1, GL_COLOR_ATTACHMENT2, GL_COLOR_ATTACHMENT3, GL_COLOR_ATTACHMENT4, GL_COLOR_ATTACHMENT5 };
		if (_renderDeferred)
		{
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_texture, 0);
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT1, GL_TEXTURE_2D, camera->render_normal_glass, 0);
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT2, GL_TEXTURE_2D, camera->render_normal, 0);
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT3, GL_TEXTURE_2D, camera->render_ambient, 0);
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT4, GL_TEXTURE_2D, camera->render_specular, 0);
			glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, camera->render_depth);

			glDrawBuffers(5, attachments);
		}
		else
		{
			glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_texture1, 0);
			glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, camera->render_depth);
			glDrawBuffers(1, attachments);
		}
	}

    glc(glClearColor(0.0,0.0,0.0,0.0));
    glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
}

void sRenderLightsUniform(sScene* scene, GLuint program)
{
	uint32_t spots=0,points=0,shadows=10;
	//uint32_t j = 0;
	for (uint16_t i=0;i<scene->lights_count;i++)
	{
		//if (scene->lights[i]->color.r==0 && scene->lights[i]->color.g==0 && scene->lights[i]->color.b==0) continue;
		laType mat = scene->lights[i]->transform_global;
		laType light_inv = Inverted(mat);
		light_inv = Mul(scene->lights[i]->projection, light_inv);

		//if (scene->lights[i]->shadow && scene->lights[i]->type!=S_SUN && !sObjectCullDot(object,(sCamera*)scene->lights[i])) continue;
		if (scene->lights[i]->type==S_SPOT)
		{
			sprintf(buff,"lSpots[%d].position",spots);
			glc(S_MATERIAL_Uniform3f(program,buff,scene->lights[i]->transform_global.a[3],scene->lights[i]->transform_global.a[7],scene->lights[i]->transform_global.a[11]));
			sprintf(buff,"lSpots[%d].direction",spots);
			glc(S_MATERIAL_Uniform3f(program,buff,scene->lights[i]->transform_global.a[2],scene->lights[i]->transform_global.a[6],scene->lights[i]->transform_global.a[10]));

			glc(sMaterialStructfv(program,"lSpots","color",spots,&scene->lights[i]->color,4));
			glc(sMaterialStructLA(program,"lSpots","itransform",spots,&light_inv));
			glc(sMaterialStructfv(program,"lSpots","inner",spots,&scene->lights[i]->inner,1));
			glc(sMaterialStructfv(program,"lSpots","outer",spots,&scene->lights[i]->outer,1));
			glc(sMaterialStructfv(program,"lSpots","zFar",spots,&scene->lights[i]->zFar,1));
			glc(sMaterialStructfv(program,"lSpots","zNear",spots,&scene->lights[i]->zNear,1));
			glc(sMaterialStructfv(program,"lSpots","FOV",spots,&scene->lights[i]->FOV,1));
			sprintf(buff,"lSpotShadowMaps[%d]",spots);
			glc(sMaterialTexture(program,buff,scene->lights[i]->render_texture, shadows));
			spots++;
		}
		else if (scene->lights[i]->type==S_POINT)
		{
			sprintf(buff,"lPoints[%d].position",points);
			glc(S_MATERIAL_Uniform3f(program,buff,scene->lights[i]->transform_global.a[3],scene->lights[i]->transform_global.a[7],scene->lights[i]->transform_global.a[11]));
			sprintf(buff,"lPoints[%d].color",points);
			glc(sMaterialUniformfv(program,buff,&scene->lights[i]->color,4));
			points++;
		}
		else if (scene->lights[i]->type==S_SUN)
		{
			glc(S_MATERIAL_Uniform3f(program,"lSun.direction",scene->lights[i]->transform_global.a[2],scene->lights[i]->transform_global.a[6],scene->lights[i]->transform_global.a[10]));
			glc(sMaterialUniformfv(program,"lSun.color",&scene->lights[i]->color,4));
			glc(sMaterialUniformfv(program,"lSun.itransform",&light_inv,16));
			glc(sMaterialUniformfv(program,"lSun.zFar",&scene->lights[i]->zFar,1));
			glc(sMaterialUniformfv(program,"lSun.zNear",&scene->lights[i]->zNear,1));
			glc(sMaterialTexture(program,"lSunShadowMap",scene->lights[i]->render_texture,shadows));
		}
		shadows += scene->lights[i]->shadow;
	}
	sMaterialUniformiv(program,"lSpotCount",&spots,1);
	sMaterialUniformiv(program,"lPointCount",&points,1);
}

void sRenderShading(sCamera* camera)
{
    sScene *scene = camera->scene;
	GLuint attachment[] = { GL_COLOR_ATTACHMENT0, GL_COLOR_ATTACHMENT1 };

    if (!camera->filters[1])
    {
    	glc(glBindFramebuffer(GL_FRAMEBUFFER,0));
    }
    else
    {
    	glc(glBindFramebuffer(GL_FRAMEBUFFER,camera->render_fb));
        if (_renderDeferred)
        {
            glc(glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_result, 0));
            glc(glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT1, GL_TEXTURE_2D, camera->render_texture1, 0));
            glc(glDrawBuffers(2, attachment));
        }
        else
        {
            glc(glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, camera->render_result, 0));
            glc(glDrawBuffers(1, attachment));
        }
    }

    /* Deferred shading */
    glc(glClearColor(0.0,0.0,0.0,0.0));
    glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
    glc(useProgram(camera->filters[0]->program));

	if (_renderDeferred)
	{
		glc(sMaterialTexture(camera->filters[0]->program,"gTextures",camera->render_texture,0));
		glc(sMaterialTexture(camera->filters[0]->program,"gAmbient",camera->render_ambient,1));
		glc(sMaterialTexture(camera->filters[0]->program,"gMasks",camera->render_specular,2));
		glc(sMaterialTexture(camera->filters[0]->program,"gNormals",camera->render_normal,3));
		glc(sMaterialTexture(camera->filters[0]->program,"gNormalsGlass",camera->render_normal_glass,4));
		glc(sMaterialTexture(camera->filters[0]->program,"gVectors",camera->render_vectors,5));
		glc(sMaterialTextureArray(camera->filters[0]->program,"gNoise",camera->noise,6));
	}
	else
	{
		glc(sMaterialTexture(camera->filters[0]->program,"rendered",camera->render_texture1,0));
	}
	
	if (scene->cubemap)
	{
		glc(sMaterialTextureCube(camera->filters[0]->program,"cubemap",scene->cubemap->ID,9));
	}

    glc(sMaterialUniformf(camera->filters[0]->program,"FPS",1.0/sGetFrameTime()));

    camera->projection = Perspective(camera->width,camera->height,camera->zFar,camera->zNear,camera->FOV);
	camera->viewProjection = Mul(camera->projection,Inverted(sCameraGetTransform(camera)));

    laType inverted_camera = Inverted(sCameraGetTransform(camera));
    laType inverted_camera_projection = Inverted(camera->projection);

    glc(sMaterialUniformLA(camera->filters[0]->program,"camera_transform", sCameraGetTransformLink(camera)));
    glc(sMaterialUniformLA(camera->filters[0]->program,"camera_inverted", &inverted_camera));
    glc(sMaterialUniformLA(camera->filters[0]->program,"projection", &camera->projection));
    glc(sMaterialUniformLA(camera->filters[0]->program,"projection_inv", &inverted_camera_projection));
    glc(sMaterialUniformf(camera->filters[0]->program,"width", camera->width));
    glc(sMaterialUniformf(camera->filters[0]->program,"height",camera->height));
    glc(sMaterialUniformf(camera->filters[0]->program,"zFar",  camera->zFar));
    glc(sMaterialUniformf(camera->filters[0]->program,"zNear", camera->zNear));
    glc(sMaterialUniformf(camera->filters[0]->program,"Angle", camera->FOV));

    if (camera->name[0]=='c' && _renderDeferred)
    {
    	sRenderLightsUniform(scene, camera->filters[0]->program);
    }

    glc(glBindBuffer(GL_ARRAY_BUFFER,camera->render_plane.VBO));
    glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,camera->render_plane.IBO));

    glc(glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 14 * sizeof(GLfloat), (GLvoid*)0));
    glc(glEnableVertexAttribArray(0));
    glc(glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 14 * sizeof(GLfloat), (GLvoid*)12));
    glc(glEnableVertexAttribArray(1));
    glc(glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, 14 * sizeof(GLfloat), (GLvoid*)24));
    glc(glEnableVertexAttribArray(2));
    glc(glVertexAttribPointer(3, 3, GL_FLOAT, GL_FALSE, 14 * sizeof(GLfloat), (GLvoid*)32));
    glc(glEnableVertexAttribArray(3));
    glc(glVertexAttribPointer(4, 3, GL_FLOAT, GL_FALSE, 14 * sizeof(GLfloat), (GLvoid*)44));
    glc(glEnableVertexAttribArray(4));

    sShaderValidate();
    glc(glDrawElements(GL_TRIANGLES,camera->render_plane.ind_count,0x1401+sizeof(index_t),BUFFER_OFFSET(0)));

    if (camera->filters[1])
    {
		for (int i=1;i<7 && camera->filters[i];i++)
		{
			GLuint texture  = (i&1) ? camera->render_texture2 : camera->render_texture1;
			GLuint rendered = (i&1) ? camera->render_texture1 : camera->render_texture2;
			useProgram(0);
			if (!camera->filters[i+1])
			{
				glc(glBindFramebuffer(GL_FRAMEBUFFER,0));
			}
			else
			{
				glc(glBindFramebuffer(GL_FRAMEBUFFER,camera->render_fb));
				glc(glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, texture, 0));
				glc(glDrawBuffers(1, attachment));
			}

			glClear(GL_DEPTH_BUFFER_BIT | GL_COLOR_BUFFER_BIT);

			GLuint program = camera->filters[i]->program;
			glc(useProgram(program));

			glc(glBindTexture(GL_TEXTURE_2D, rendered));
			if ((1<<i)&camera->mipmap_layers)
			{
				glc(glBindTexture(GL_TEXTURE_2D, rendered));
				glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
		    	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
				glc(glGenerateMipmap(GL_TEXTURE_2D));
			}
		    else
		    {
				glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
				glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR));
		    }

			for (uint32_t l=0;l<scene->lights_count;l++)
			if (scene->lights[l]->type == S_SUN)
			{
				//printf("lSunDirection\n");
				laType mat = scene->lights[l]->transform_global;
				laType light_inv = Inverted(mat);
				light_inv = Mul(scene->lights[l]->projection, light_inv);
				glc(S_MATERIAL_Uniform3f(program,"lSunDirection",scene->lights[l]->transform_global.a[2],scene->lights[l]->transform_global.a[6],scene->lights[l]->transform_global.a[10]));
				glc(S_MATERIAL_Uniform3f(program,"lSunPosition",scene->lights[l]->transform_global.a[3],scene->lights[l]->transform_global.a[7],scene->lights[l]->transform_global.a[11]));
				glc(sMaterialUniformfv(program,"lSunColor", &scene->lights[l]->color, 3));

				glc(sMaterialUniformfv(program,"lSunTransform",light_inv.a,16));
				glc(sMaterialUniformfv(program,"lSunFar",&scene->lights[l]->zFar,1));
				glc(sMaterialUniformfv(program,"lSunNear",&scene->lights[l]->zNear,1));

				glc(sMaterialTexture(program,"lSunShadowMap",scene->lights[l]->render_texture,15));
				break;
			}
			
			glc(sMaterialTexture(program,"filtered",rendered,9));
			glc(sMaterialTexture(program,"original",camera->render_result,10));
			
			if (_renderDeferred)
			{
				glc(sMaterialTexture(program,"gTextures",camera->render_texture,0));
				glc(sMaterialTexture(program,"gAmbient",camera->render_ambient,1));
				glc(sMaterialTexture(program,"gMasks",camera->render_specular,2));
				glc(sMaterialTexture(program,"gNormals",camera->render_normal,3));
				glc(sMaterialTexture(program,"gNormalsGlass",camera->render_normal_glass,4));
				glc(sMaterialTexture(program,"gVectors",camera->render_vectors,5));
				glc(sMaterialTextureArray(program,"gNoise",camera->noise,6));
			}
			if (scene->cubemap)
			{
				glc(sMaterialTextureCube(program,"cubemap",scene->cubemap->ID,11));
			}

			glc(sMaterialUniformf(program,"FPS",1.0/sGetFrameTime()));

			glc(sMaterialUniformLA(program,"camera_transform", sCameraGetTransformLink(camera)));
			glc(sMaterialUniformLA(program,"projection", &camera->projection));
			glc(sMaterialUniformLA(program,"projection_inv", &inverted_camera_projection));
			glc(sMaterialUniformLA(program,"camera_inverted", &inverted_camera));
			glc(sMaterialUniformf(program,"width", camera->width));
			glc(sMaterialUniformf(program,"height",camera->height));
			glc(sMaterialUniformf(program,"zFar",  camera->zFar));
			glc(sMaterialUniformf(program,"zNear", camera->zNear));
			glc(sMaterialUniformf(program,"Angle", camera->FOV));
			glc(sMaterialUniformf(program,"time", sEngineGetTime()));

			/*glValidateProgram(program);
			int loglen,testVal;
			glc(glGetProgramInfoLog(program, 10240, &loglen, shader_log_buffer));
			glc(glGetProgramiv(program, GL_VALIDATE_STATUS, &testVal));
			if(testVal == GL_FALSE)
			{
				puts(shader_log_buffer);
			}*/
			sShaderValidate();
			glc(glDrawElements(GL_TRIANGLES,camera->render_plane.ind_count,0x1401+sizeof(index_t),BUFFER_OFFSET(0)));
		}
    }
}

float rotation = 0.0;

sMesh* activeMesh = 0;

laType sCameraGetTransform(sCamera* camera)
{
	if (camera->view_point)
	{
		return camera->view_point->transform_global;
	}
	else
	{
		return camera->transform_global;
	}
}
laType sCameraGetTransformPrev(sCamera* camera)
{
	if (camera->view_point)
	{
		return camera->view_point->transform_global_previous;
	}
	else
	{
		return camera->transform_global_previous;
	}
}

laType *sCameraGetTransformLink(sCamera* camera)
{
	if (camera->view_point)
	{
		return &camera->view_point->transform_global;
	}
	else
	{
		return &camera->transform_global;
	}
}
laType *sCameraGetTransformPrevLnk(sCamera* camera)
{
	if (camera->view_point)
	{
		return &camera->view_point->transform_global_previous;
	}
	else
	{
		return &camera->transform_global_previous;
	}
}

_Bool sObjectCullDot(sObject* object,sCamera* camera)
{
    float bbox[3] = {fabs(object->mesh->bounding_box.a[0]),
        fabs(object->mesh->bounding_box.a[1]),
        fabs(object->mesh->bounding_box.a[2])};
        //laType vert = Mul(object->mesh->bounding_box,object->transform_global);
        laType MVP = Mul(camera->viewProjection,object->transform_global);
        float scale = 2.0;
        laType box[] =
        {
            Mul(Vector4(-bbox[0]/scale,-bbox[1]/scale,-bbox[2]/scale,1.0),MVP),
            Mul(Vector4( bbox[0]/scale,-bbox[1]/scale,-bbox[2]/scale,1.0),MVP),
            Mul(Vector4(-bbox[0]/scale, bbox[1]/scale,-bbox[2]/scale,1.0),MVP),
            Mul(Vector4( bbox[0]/scale, bbox[1]/scale,-bbox[2]/scale,1.0),MVP),
            Mul(Vector4(-bbox[0]/scale,-bbox[1]/scale, bbox[2]/scale,1.0),MVP),
            Mul(Vector4( bbox[0]/scale,-bbox[1]/scale, bbox[2]/scale,1.0),MVP),
            Mul(Vector4(-bbox[0]/scale, bbox[1]/scale, bbox[2]/scale,1.0),MVP),
            Mul(Vector4( bbox[0]/scale, bbox[1]/scale, bbox[2]/scale,1.0),MVP)
        };
        //laType scale;
        _Bool out = 0;
        for (uint32_t i=0;i<8;i++)
        {
            box[i].a[0]/=fabs(box[i].a[3]);
            box[i].a[1]/=fabs(box[i].a[3]);
            box[i].a[2]/=box[i].a[3];
        }
        _Bool ortho = camera->name[0]=='l' && ((sLight*)camera)->type == S_SUN;

        out = 1; for (uint32_t i=0;i<8;i++) {out &= (box[i].a[0]<-1.0 && box[i].a[1]<-1.0);} if (out) return 0;
        out = 1; for (uint32_t i=0;i<8;i++) {out &= (box[i].a[0]<-1.0 && box[i].a[1]> 1.0);} if (out) return 0;
        out = 1; for (uint32_t i=0;i<8;i++) {out &= (box[i].a[0]> 1.0 && box[i].a[1]> 1.0);} if (out) return 0;
        out = 1; for (uint32_t i=0;i<8;i++) {out &= (box[i].a[0]> 1.0 && box[i].a[1]<-1.0);} if (out) return 0;
        out = 1; for (uint32_t i=0;i<8;i++) {out &= box[i].a[2]<-ortho;} if (out) return 0;
        out = 1; for (uint32_t i=0;i<8;i++) {out &= box[i].a[2]> 1.0;} if (out) return 0;
        return 1;
}

static sCamera* _pcamera;
void sRenderDrawObject(sObject* object,sCamera* camera, _Bool cull_invisible)
{
    sMesh* mesh = object->name[0]=='b' ? ((sBone*)object)->mesh : object->mesh;
    //fprintf(LOGOUT, "%s object\n", object->name);
    if ((!mesh || !sObjectCullDot(object,camera)) && cull_invisible)
    {
        return;
    }

    for (sObject *obj=object;obj;obj=obj->parent)
    {
    	if (obj->hidden) return;
    }
    //fprintf(LOGOUT, "%s draw\n", object->name);
    sScene* scene = object->scene;
    sSkeleton* skeleton = object->skeleton;
    sMaterial* material = mesh->material;
    laType transform;

    GLuint program = mesh->material->shader;
    useProgram(mesh->material->shader);

    laType camt;

	camt = Inverted(sCameraGetTransform(camera));
	transform = object->transform_global;
    glc(sMaterialUniformLA(program,"camera_inverted",&camt));
    glc(sMaterialUniformLA(program,"camera_transform",sCameraGetTransformLink(camera)));
    glc(sMaterialUniformLA(program,"transform",&transform));
    glc(sMaterialUniformLA(program,"projection",&camera->projection));
    camt = Inverted(camera->projection);
    glc(sMaterialUniformLA(program,"projection_inverted",&camt));

    if (_renderVectors)
    {
    	camt = Inverted(sCameraGetTransformPrev(camera));
    	transform = object->transform_global_previous;
        glc(sMaterialUniformLA(program,"camera_inverted_pd",&camt));
        glc(sMaterialUniformLA(program,"transform_pd",&transform));
    }

    if (mesh->deformed)
    {
    	uint32_t wi;
		for (wi=0; mesh->bones_indices[wi]!=0xFFFF && wi<128; wi++)
		{
			laType bone;
			uint32_t i = mesh->bones_indices[wi];
			bone = Mulmc(skeleton->bones[i].transform_global,mesh->link_matrix[i]);
			sprintf(buff,"bones[%d]",wi);
			//puts(buff);
			glc(sMaterialUniformfv(program,buff,bone.a,12));

			if (_renderVectors)
			{
				bone = Mulmc(skeleton->bones[i].transform_global_previous,mesh->link_matrix[i]);
				sprintf(buff,"bones_pd[%d]",wi);
				glc(sMaterialUniformfv(program,buff,bone.a,12));
			}
		}
		//printf("%s %s\n",mesh->name+1, buff);
    }

    if (activeMesh == mesh && _pcamera == camera)
    {
        goto drawing;
    }
    activeMesh = mesh;

    uint32_t proj = !(camera->name[0]=='l' && ((sLight*)camera)->type==S_SUN);
    glc(sMaterialUniformiv(program,"render_projection", &proj, 1));

    {
        glc(sMaterialUniformf(program,"width", camera->width));
        glc(sMaterialUniformf(program,"height",camera->height));
        glc(sMaterialUniformf(program,"zFar",  camera->zFar));
        glc(sMaterialUniformf(program,"zNear", camera->zNear));
        glc(sMaterialUniformf(program,"Angle", camera->FOV));
    }

    glc(sMaterialUniformfv(program,"material_diffuse",&mesh->material->diffuse,3));
    glc(sMaterialUniformfv(program,"material_specular",&mesh->material->specular,3));
    glc(sMaterialUniformi(program,"material_dtex",  material->diffuse_texture!=0));
    glc(sMaterialUniformi(program,"material_stex",  material->specular_texture!=0));
    glc(sMaterialUniformi(program,"material_htex",  material->height_texture!=0));

    glc(sMaterialUniformf(program,"material_glow",material->glow));
    glc(sMaterialUniformf(program,"material_wet",material->wet));
    //glc(sMaterialUniformi(program,"material_rtex",  material->reflection_cubemap!=0));
    glc(sMaterialUniformi(program,"material_ltex",  material->lightmap_texture!=0));

    float sas[] = {material->tdx,material->tdy};
    glc(sMaterialUniformfv(program,"texture_displacement",sas,2));
    glc(sMaterialUniformfv(program,"transparency",&material->transparency,1));
    glc(sMaterialUniformf(program,"height_scale", mesh->material->height_scale));
    if (material->diffuse_texture!=0)
    {
        glc(sMaterialTexture(program,"diffuse_map",material->diffuse_texture->ID,0));
    }
    else
    {
        glc(sMaterialTexture(program,"diffuse_map",0,0));
    }
    if (material->specular_texture!=0)
    {
        glc(sMaterialTexture(program,"specular_map",material->specular_texture->ID,1));
    }
    else
    {
        glc(sMaterialTexture(program,"specular_map",0,1));
    }
    if (material->height_texture!=0)
    {
        float n_size[] = {material->height_texture->width,material->height_texture->height};
        glc(sMaterialTexture(program,"normal_map",material->height_texture->ID,2));
        glc(sMaterialUniformfv(program,"normal_map_size",n_size,2));
    }
    else
    {
        glc(sMaterialTexture(program,"normal_map",0,2));
    }
    if (material->lightmap_texture!=0)
    {
        glc(sMaterialTexture(program,"light_map",material->lightmap_texture->ID,3));
    }
    else
    {
        glc(sMaterialTexture(program,"light_map",0,3));
    }
    float refl;
    if (material->reflection_cubemap!=0)
    {
        glc(sMaterialTextureCube(program,"reflection_map",material->reflection_cubemap->ID,4));
        refl = 1.0;
    }
    else
    {
        glc(sMaterialTextureCube(program,"reflection_map",0,4));
        refl = 0.0;
    }
    glc(sMaterialUniformfv(program,"reflection_coeff",&refl,1));
	
	if (!_renderDeferred)
	{
		uint32_t spots=0,points=0,shadows=0;
        //uint32_t j = 0;
        for (uint16_t i=0;i<scene->lights_count;i++)
        {
        	//if (scene->lights[i]->color.r==0 && scene->lights[i]->color.g==0 && scene->lights[i]->color.b==0) continue;
            laType mat = scene->lights[i]->transform_global;
            laType light_inv = Inverted(mat);
            light_inv = Mul(scene->lights[i]->projection, light_inv);

            //if (scene->lights[i]->shadow && scene->lights[i]->type!=S_SUN && !sObjectCullDot(object,(sCamera*)scene->lights[i])) continue;
            if (scene->lights[i]->type==S_SPOT)
            {
            	sprintf(buff,"lSpots[%d].position",spots);
                glc(S_MATERIAL_Uniform3f(program,buff,scene->lights[i]->transform_global.a[3],scene->lights[i]->transform_global.a[7],scene->lights[i]->transform_global.a[11]));
            	sprintf(buff,"lSpots[%d].direction",spots);
                glc(S_MATERIAL_Uniform3f(program,buff,scene->lights[i]->transform_global.a[2],scene->lights[i]->transform_global.a[6],scene->lights[i]->transform_global.a[10]));

                glc(sMaterialStructfv(program,"lSpots","color",spots,&scene->lights[i]->color,4));
                glc(sMaterialStructLA(program,"lSpots","itransform",spots,&light_inv));
                glc(sMaterialStructfv(program,"lSpots","inner",spots,&scene->lights[i]->inner,1));
                glc(sMaterialStructfv(program,"lSpots","outer",spots,&scene->lights[i]->outer,1));
                glc(sMaterialStructfv(program,"lSpots","zFar",spots,&scene->lights[i]->zFar,1));
                glc(sMaterialStructfv(program,"lSpots","zNear",spots,&scene->lights[i]->zNear,1));
                glc(sMaterialStructfv(program,"lSpots","FOV",spots,&scene->lights[i]->FOV,1));
            	sprintf(buff,"lSpotShadowMaps[%d]",spots);
                glc(sMaterialTexture(program,buff,scene->lights[i]->render_texture,15 - shadows));
            	spots++;
            }
            else if (scene->lights[i]->type==S_POINT)
            {
            	sprintf(buff,"lPoints[%d].position",points);
                glc(S_MATERIAL_Uniform3f(program,buff,scene->lights[i]->transform_global.a[3],scene->lights[i]->transform_global.a[7],scene->lights[i]->transform_global.a[11]));
            	sprintf(buff,"lPoints[%d].color",points);
                glc(sMaterialUniformfv(program,buff,&scene->lights[i]->color,4));
            	points++;
            }
            else if (scene->lights[i]->type==S_SUN)
            {
            	glc(S_MATERIAL_Uniform3f(program,"lSun.direction",scene->lights[i]->transform_global.a[2],scene->lights[i]->transform_global.a[6],scene->lights[i]->transform_global.a[10]));
                glc(sMaterialUniformfv(program,"lSun.color",&scene->lights[i]->color,4));
                glc(sMaterialUniformfv(program,"lSun.itransform",&light_inv,16));
                glc(sMaterialUniformfv(program,"lSun.zFar",&scene->lights[i]->zFar,1));
                glc(sMaterialUniformfv(program,"lSun.zNear",&scene->lights[i]->zNear,1));
                glc(sMaterialTexture(program,"lSunShadowMap",scene->lights[i]->render_texture,15 + shadows));
            }
            shadows += scene->lights[i]->shadow;
        }
        sMaterialUniformiv(program,"lSpotCount",&spots,1);
        sMaterialUniformiv(program,"lPointCount",&points,1);
	}
    glc(glBindBuffer(GL_ARRAY_BUFFER,mesh->VBO));
    glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,mesh->IBO));

    glc(glEnableVertexAttribArray(0));
	glBindAttribLocation(program, 0, "pos");
    glc(glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)0));

    glc(glEnableVertexAttribArray(2));
	glBindAttribLocation(program, 2, "uv");
    glc(glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)24));

    if (camera->name[0]=='c')
    {
        glc(glEnableVertexAttribArray(1));
    	glBindAttribLocation(program, 1, "nor");
        glc(glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)12));
        glc(glEnableVertexAttribArray(3));
    	glBindAttribLocation(program, 3, "bin");
        glc(glVertexAttribPointer(3, 3, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)32));
        glc(glEnableVertexAttribArray(4));
    	glBindAttribLocation(program, 4, "tang");
        glc(glVertexAttribPointer(4, 3, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)44));
        if (mesh->uv2)
        {
            glc(glEnableVertexAttribArray(5));
            glc(glVertexAttribPointer(5, 2, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)56+mesh->deformed*12));
        }
        else
        {
            glc(glEnableVertexAttribArray(5));
            glc(glVertexAttribPointer(5, 2, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)24));
        }
    }
    if (mesh->deformed)
    {
        glc(glEnableVertexAttribArray(6));
        glc(glVertexAttribPointer(6, 3, GL_FLOAT, GL_FALSE, (14 + mesh->deformed*3+mesh->uv2*2) * sizeof(GLfloat), (GLvoid*)56));
    }
    drawing:
    glc(glDrawElements(GL_TRIANGLES,mesh->ind_count,0x1401+sizeof(index_t),BUFFER_OFFSET(0)));
    _pcamera = camera;
}

extern struct
{
	double animations;
	double scripts;
	double physics;
	double placing;
	double shadows;
	double rasterizing;
	double shading;
	double interface;
	double buffers;
} sProfilingTimers;

void sRenderClear(float r,float g, float b, float a)
{
	glc(glBindFramebuffer(GL_FRAMEBUFFER,0));
	glc(glClearColor(r, g, b, a));
	glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
}

void sRenderDrawScene(sScene *scene)
{
	//lights shadows
	double timer = glfwGetTime();
	for (index_t l=0;l<scene->lights_count;l++)
	{
		if (!scene->lights[l]->shadow || (scene->lights[l]->color.r==0 && scene->lights[l]->color.g==0 && scene->lights[l]->color.b==0)) continue;
		sCameraBindFB(((sCamera*)scene->lights[l]),0);
		glc(glClearColor(0, 0, 0, 0));
		glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
		glDisable(GL_CULL_FACE);
		for (size_s i=0;i<scene->objects_count;i++)
		{
			sObject* obj = scene->objects[i];
			if (!obj->mesh) continue;
			if (obj->mesh->deformed)
				obj->mesh->material->shader = scene->shader_list[3]->program;
			else
				obj->mesh->material->shader = scene->shader_list[2]->program;

			sRenderDrawObject(obj,((sCamera*)scene->lights[l]), 1);
		}
	}
	//glFinish();
	sProfilingTimers.shadows = glfwGetTime() - timer;
	/////////////////

	timer = glfwGetTime();

	sCameraClearRenderTargets(&scene->camera);
	sCameraBindFB(&scene->camera,1);
	glc(glClearColor(0.0,0.0,0.0,0.0));
	glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));

	//Solid objectsold
	glEnable(GL_CULL_FACE);
	glDisable(GL_BLEND);
	glDepthFunc(GL_LEQUAL);
	glEnable(GL_DEPTH_TEST);
	glAlphaFunc (GL_ALWAYS, 0.5);
	glDepthMask( GL_TRUE );
	//glDepthMask( GL_FALSE );
	//glEnable(GL_DEPTH_WRITEMASK);
	for (size_s i=0;i<scene->objects_count;i++)
	{
		sObject* obj = scene->objects[i];
		if (!obj->mesh) continue;
		if (obj->mesh->material->transparency==0.0)
		{
			if (obj->mesh->deformed)
				obj->mesh->material->shader = scene->shader_list[1]->program;
			else
				obj->mesh->material->shader = scene->shader_list[0]->program;
			sRenderDrawObject(obj,&scene->camera, 1);
		}
	}
	////////////////////
	// Transparent objects
	sCameraBindFB(&scene->camera, 0);
	glc(glDisable(GL_CULL_FACE));
	glc(glDisable(GL_ALPHA_TEST));
	glc(glDisable(GL_BLEND));

	for (size_s i=0;i<scene->objects_count;i++)
	{
		sObject* obj = scene->objects[i];
		if (!obj->mesh) continue;
		if (obj->mesh->material->transparency==0.0) continue;
		if (obj->mesh->deformed)
			obj->mesh->material->shader = scene->shader_list[1]->program;
		else
			obj->mesh->material->shader = scene->shader_list[0]->program;
		sRenderDrawObject(obj,&scene->camera, 1);
	}
	/////////////////////
#ifdef DRAW_BONES
	{
		sObject* bone = 0;
		for (uint32_t i=0; i<scene->objects_count; i++)
		{
			if (strcmp(scene->objects[i]->name,"obone")==0)
			{
				bone = scene->objects[i];
				break;
			}
		}
		if (bone)
		{
			glClear(GL_DEPTH_BUFFER_BIT);
			for (uint32_t i=0; i<scene->skeletons_count; i++)
			{
				sSkeleton* skel = scene->skeletons[i];
				//puts(skel->name);
				for (uint32_t b=0; b<skel->bone_count; b++)
				{
					bone->transform_global = skel->bones[b].transform_global;
					if (skel->bones[b].child_count)
					{
						laType vect = sObjectGetVectorTo(bone, skel->bones[b].children[0]);
						float l = Length(vect);
						bone->transform_global.a[ 0] *= l;
						bone->transform_global.a[ 1] *= l;
						bone->transform_global.a[ 2] *= l;
						bone->transform_global.a[ 4] *= l;
						bone->transform_global.a[ 5] *= l;
						bone->transform_global.a[ 6] *= l;
						bone->transform_global.a[ 8] *= l;
						bone->transform_global.a[ 9] *= l;
						bone->transform_global.a[10] *= l;
					}
					else
					{
						bone->transform_global.a[ 0] *= 0.02;
						bone->transform_global.a[ 1] *= 0.02;
						bone->transform_global.a[ 2] *= 0.02;
						bone->transform_global.a[ 4] *= 0.02;
						bone->transform_global.a[ 5] *= 0.02;
						bone->transform_global.a[ 6] *= 0.02;
						bone->transform_global.a[ 8] *= 0.02;
						bone->transform_global.a[ 9] *= 0.02;
						bone->transform_global.a[10] *= 0.02;
					}
					bone->transform = bone->transform_global;
					sRenderDrawObject(bone, &scene->camera, 0);
				}
			}
		}
	}
#endif

	// All objects again for stretched vector masks //
	//_renderVectors = 1;
	if (_renderVectors)
		sCameraBindVectorsFB(&scene->camera);
	for (size_s i=0;i<scene->objects_count;i++)
	{
		sObject* obj = scene->objects[i];
		if (obj->mesh && _renderVectors)
		{
			if (obj->mesh->deformed)
				obj->mesh->material->shader = scene->shader_list[5]->program;
			else
				obj->mesh->material->shader = scene->shader_list[4]->program;
			sRenderDrawObject(obj,&scene->camera, 1);
		}
		obj->transform_global_previous = obj->transform_global;
	}
	for (size_s i=0;i<scene->skeletons_count;i++)
	{
		sSkeleton* obj = scene->skeletons[i];
		for (uint32_t j=0;j<obj->bone_count;j++)
		{
			obj->bones[j].transform_global_previous = obj->bones[j].transform_global;
		}
		obj->transform_global_previous = obj->transform_global;
	}

	scene->camera.transform_global_previous = scene->camera.transform_global;
	sCameraBindFB(&scene->camera, 1);

	glDisable(GL_BLEND);
	glEnable(GL_CULL_FACE);
	glDepthFunc(GL_LEQUAL);
	glEnable(GL_DEPTH_TEST);
	glAlphaFunc (GL_ALWAYS, 0.5);
	glDepthMask( GL_TRUE );
	//glFinish();
	glFlush();
	sProfilingTimers.rasterizing = glfwGetTime() - timer;

	timer = glfwGetTime();
	sRenderShading(&scene->camera);
	//glFinish();
	glFlush();
	sProfilingTimers.shading = glfwGetTime() - timer;
}
