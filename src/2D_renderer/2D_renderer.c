/*
 * 2D_renderer.c
 *
 *  Created on: 1 авг. 2019 г.
 *      Author: ivan
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "engine.h"
#include "2D_renderer/2D_renderer.h"
#include "2D_renderer/forms.h"

fForm**  forms = 0;
int forms_count = 0;
fElement** elements = 0;
int elements_count = 0;

sShader _font_shader;
sShader _rectangle_shader;
sMesh   _rectangle;
uint16_t text_width, text_htight;
uint32_t text_scrolling;
sTexture default_font;

int ID_counter = 1;

static sVertex rectangle_points[] =
{{ 0.0, 0.0, 0.0,	0.0, 0.0, 1.0,	0.0, 0.0},
 { 0.0, 1.0, 0.0,	0.0, 0.0, 1.0,	0.0, 1.0},
 { 1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	1.0, 1.0},
 { 1.0, 0.0, 0.0,	0.0, 0.0, 1.0,	1.0, 0.0}};

static index_t _rectangle_indices[] = {0,3,2,2,1,0};

void listPushBack(void** list, int* length, void* item, int size)
{
	*list = sRealloc(*list, (1 + *length)*size);
	memmove((*list) + (*length) * size, item, size);
	(*length)++;
}

void listPop(void** list, int* length, int index)
{
	if (*length == 1)
	{
		sFree(*list);
		*length = 0;
		*list = 0;
	}
	else
	{
		void** ptr = (void**)*list;
		for (int i=index; i<(*length)-1; i++)
		{
			ptr[i] = ptr[i+1];
		}
		*list = sRealloc(*list, sizeof(void*)*((*length)-1));
		(*length)--;
	}
}

void listPopPointer(void** list, int* length, void* pointer)
{
	void*** ptr = (void***)list;
	int index = listIndexOf(*ptr, *length, pointer);
	for (index = 0; index<*length && (*ptr)[index]!=pointer; index++);
	if (index==-1)
	{
		return;
	}
	listPop(list, length, index);
}

int listIndexOf(void** list, int length, void* item)
{
	for (int i=0;i<length;i++)
	{
		if (list[i] == item)
		{
			return i;
		}
	}
	return -1;
}

void listSwapElements(void** list, int ind1, int ind2)
{
	void* w = list[ind1];
	list[ind1] = list[ind2];
	list[ind2] = w;
}

void formsInit(void)
{
	sTextureLoadDDS(&default_font, "data/fonts/default.dds");

	glc(glGenBuffers(1, &_rectangle.VBO));
	glc(glGenBuffers(1, &_rectangle.IBO));

	glc(glBindBuffer(GL_ARRAY_BUFFER, _rectangle.VBO));
	glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, _rectangle.IBO));

	glc(glBufferData(GL_ARRAY_BUFFER, sizeof(rectangle_points), rectangle_points, GL_STATIC_DRAW));
	glc(glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(_rectangle_indices), _rectangle_indices, GL_STATIC_DRAW));

	sLoadVertexFromFile(&_rectangle_shader,"data/shaders/UI2/rectangle_vertex.glsl");
    sLoadFragmentFromFile(&_rectangle_shader,"data/shaders/UI2/rectangle_fragment.glsl");
	sShaderMake(&_rectangle_shader);

	sLoadVertexFromFile(&_font_shader,"data/shaders/UI2/text_plane.glsl");
	sLoadFragmentFromFile(&_font_shader,"data/shaders/UI2/text.glsl");
	sShaderMake(&_font_shader);

	forms  = 0;
	forms_count = 0;
	elements = 0;
	elements_count = 0;
}

void fFormsClear(void)
{
	while (forms_count)
	{
		fFormDelete(forms[0]);
	}
	while (elements_count)
	{
		fElementDelete(elements[0]);
	}
	sTextureFree(&default_font);
	glDeleteBuffers(1,&_rectangle.VBO);
	glDeleteBuffers(1,&_rectangle.IBO);
	sShaderDestroy(&_font_shader);
	sShaderDestroy(&_rectangle_shader);
}

char* text = "The C programming language has a set of functions implementing. "
			 "operations on strings (character strings and byte strings) in "
			 "its standard library. Various operations, such as copying, "
			 "concatenation, tokenization and searching are supported. For "
			 "character strings, the standard library uses the convention "
			 "that strings are null-terminated:   a string of n characters is "
		  	 "represented as an array of n + 1 elements, the last of which is "
			 "a \"NUL\" character";

void *logic_thread(void *ptr);
void sMouseShow(void);
void sEngineSetSwapInterval(uint32_t interval);

void test()
{
	_renderRayTracing = 0;
	sEngineCreateWindow(800,480,0);
	sMouseShow();
	sEngineStartOpenGL();
	sEngineSetSwapInterval(1);

	glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
	glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT));

	glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
	glc(glViewport(0,0,sEngineGetWidth(),sEngineGetHeight()));
	glc(glClearColor(0.15,0.15,0.15,1.0));
	glc(glClearStencil(0x00));
	glc(glStencilMask(0xFF));
	glc(glDisable(GL_CULL_FACE));
	glc(glEnable(GL_STENCIL_TEST));

	printf("loop %lld\n", sGetAllocatedMem());

	fButtonCreate((char*)"Test", 100, 100,-1,-1, 0);
	fForm list;

	void printIned(fForm* bttn, int index)
	{
		printf("You've chosen %s\n", ((fElement*)bttn->elements[1])->text);
		fFormMarkDelete(&list);
	}
	fListConstructor(&list, 300,100, 100, 300, printIned);
	fListAddItem(&list, "kajsdhjk");
	fListAddItem(&list, "kajsdhjk");
	fListAddItem(&list, "kajsdhjk");
	fListAddItem(&list, "kajsdhjk");
	float cur_x, cur_y;
	float p_x, p_y;
	sMouseGetPosition(&cur_x, &cur_y);
	sMouseGetPosition(&p_x, &p_y);
	while (!sEngineShouldClose())
	{
		glClear(GL_COLOR_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);
		fFormsProcess();

		logic_thread(0);
		sEngineSwapBuffers();
	}
	sEngineClose();
}
