/*
 * fForm.h
 *
 *  Created on: 6 авг. 2019 г.
 *      Author: ivan
 */

#ifndef SRC_FFORM_H_
#define SRC_FFORM_H_

#include "2D_renderer.h"

typedef struct
{
	int type;
	int ID;
	_Bool delete_me;
	_Bool lock_rotation;
	_Bool visible;
	_Bool limits;
	_Bool xray;
	_Bool ghost;
	float transform[9];
	float transform_global[9];
	void* parent;
	fElement** elements;
	int elements_count;
	void** children;
	int child_count;
	int width, height;
	float xscroll,yscroll;
	void(*on_left_click)(void*);
	void(*on_right_click)(void*);
	void(*on_scroll)(void*,int);
	void(*idle)(void*);
	void(*on_left_release)(void*);
	void(*on_right_release)(void*);
	void(*on_release)(void*);
	void(*on_pointing)(void*);
	int hovered;
	void* data;
} fForm;

fForm*    fFormCreate(void); // "Конструктор" 1
void      fFormConstructor(fForm* form); // "Конструктор" 2
void      fFormRemoveParent(fForm* gp);
void      fFormMarkDelete(fForm* form);
int       fFormCheckHoverBounds(fForm* form, float x, float y);
void      fFormAddForm(fForm* form, fForm* child);
fElement* fFormAddElement(fForm* form, char* text, int font_size, float width, float height);
fForm*    fFormGetChild(fForm* form, int index);
int       fFormGetChildCount(fForm* form);
fElement* fFormGetElement(fForm* form, int index);
void      fFormDraw(fForm* form, int z);
void      fFormDelete(fForm* form); // "Деструктор"

void fFormSetVisibleBit(fForm* form, _Bool bit);
void fFormSetLimitsBit(fForm* form, _Bool bit);
void fFormSetXRayBit(fForm* form, _Bool bit);
void fFormSetGhostBit(fForm* form, _Bool bit);

_Bool fFormGetVisibleBit(fForm* form);
_Bool fFormGetLimitsBit(fForm* form);
_Bool fFormGetXRayBit(fForm* form);
_Bool fFormGetGhostBit(fForm* form);

void fFormSetWidth(fForm* form, float w);
void fFormSetHeight(fForm* form, float h);
float  fFormGetWidth(fForm* form);
float  fFormGetHeight(fForm* form);

void fFormSetIdle(fForm* form, void(*callback)(fForm*));
void fFormSetLMB(fForm* form, void(*callback)(fForm*));
void fFormSetRMB(fForm* form, void(*callback)(fForm*));
void fFormSetScroll(fForm* form, void(*callback)(fForm*,int));
void fFormSetCursorHover(fForm* form, void(*callback)(fForm*));
void fFormSetCursorLeave(fForm* form, void(*callback)(fForm*));

void fFormSetTopLayer(fForm* form);
void fFormSetBottomLayer(fForm* form);
void fFormMovLayerDown(fForm* form);
void fFormMovLayerUp(fForm* form);

void fFormApplyTransformToLocal(fForm* form);
void fFormApplyTransformToGlobal(fForm* form);
void fFormApplyTransformToChildren(fForm* form);

void fFormScrollVertical  (fForm* form, float v);
void fFormScrollHorizontal(fForm* form, float h);

void fFormSetVerticalScrolling(fForm* form, float v);
void fFormSetHorizontalScrolling(fForm* form, float h);
float fFormGetVerticalScrolling(fForm* form);
float fFormGetHorizontalScrolling(fForm* form);

void fFormSetLocalPosition(fForm* form, float x, float y);
void fFormSetGlobalPosition(fForm* form, float x, float y);
void fFormGetLocalPosition(fForm* form, float* pos);
void fFormGetGlobalPosition(fForm* form, float* pos);

void fFormTranslateGlobal(fForm* form, float x, float y);
void fFormTranslateLocal(fForm* form, float x, float y);

void fFormSetRotation(fForm* form, float x);

void fFormRotate(fForm* form, float x);
void fFormSetRotationLocal(fForm* form, float x);
void fFormSetRotationGlobal(fForm* form, float x);
float fFormGetLocalRotation(fForm* form);
float fFormGetGlobalRotation(fForm* form);

int fFormCheckHover(fForm* form, float x, float y);
void fFormsSetPostFunctionInterval(int interval);
void fFormsSetPostFunction(void(*callback)(void));
void fFormsProcess(void);

#endif /* SRC_FFORM_H_ */
