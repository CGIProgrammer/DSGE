/*
 * 2D_renderer.h
 *
 *  Created on: 6 авг. 2019 г.
 *      Author: ivan
 */

#ifndef SRC_2D_RENDERER_H_
#define SRC_2D_RENDERER_H_

#include "engine.h"
#include "stdint.h"
#include "fText.h"
#include "fForm.h"

#define pointersDiff(ptr1, ptr2) ((intptr_t)(ptr2) - (intptr_t)(ptr1))

void listPushBack(void** list, int* length, void* index, int size);
void listPop(void** list, int* length, int index);
int listIndexOf(void** list, int length, void* item);
void listSwapElements(void** list, int ind1, int ind2);
void listPopPointer(void** list, int* length, void* pointer);

enum
{
	F_TEXT,
	F_RECT,
	F_FORM,
	F_GROUP
};

extern sShader _font_shader;
extern sShader _rectangle_shader;
extern sMesh   _rectangle;
extern uint16_t text_width, text_htight;
extern uint32_t text_scrolling;
extern sTexture default_font;
extern int ID_counter;

void formsInit(void);
void fFormsClear(void);

#endif /* SRC_2D_RENDERER_H_ */
