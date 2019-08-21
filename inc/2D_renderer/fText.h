/*
 * fText.h
 *
 *  Created on: 6 авг. 2019 г.
 *      Author: ivan
 */

#ifndef SRC_FTEXT_H_
#define SRC_FTEXT_H_

typedef struct
{
	int ID;
	_Bool lock_rotation;
	_Bool visible;
	float transform[9];
	float transform_global[9];
	float layer;
	void* form;
	float text_color[4];
	float bg_color[4];
	int width, height;
	char** lines;
	int lines_count;
	char* text;
	int length;
	_Bool dynamic_carry;
	_Bool carry;
	int font_width;
	int cursor1;
	int cursor2;
	int align;
	sTexture* font;
	sTexture* bg_image;
	float uv_start[2];
	float uv_size[2];
	int hovered;
} fElement;

fElement* fElementCreate(char* text, int font_size, int width, int height, int carry);

void  fElementSetVisibleBit(fElement* element, _Bool bit);
_Bool fElementGetVisibleBit(fElement* element);

void fElementSetLockRotationBit(fElement* element, _Bool bit);
_Bool fElementGetLockRotationBit(fElement* element);

void fElementSetText(fElement* element, char* text);
int  fElementGetTextLength(fElement* element);
void fElementGetText(fElement* element, char* text, int length);
char* fElementGetTextPtr(fElement* element);
void fElementDrawText(fElement* felementt);
void fElementDrawRect(fElement* element);
void fElementDelete(fElement* element);
void fElementPrint(fElement* element);
void fElementApplyTransformToGlobal(fElement* ft);
int  fElementCheckHover(fElement* rect, float x, float y);

void fElementTranslateLocal(fElement* element, float x, float y);
void fElementTranslateGlobal(fElement* element, float x, float y);

void fElementSetLocalPosition(fElement* element, float x, float y);
void fElementSetGlobalPosition(fElement* element, float x, float y);

void fElementGetLocalPosition(fElement* element, float* x, float* y);
void fElementGetGlobalPosition(fElement* element, float* x, float* y);

float fElementGetLocalRotation(fElement* element);
void  fElementSetLocalRotation(fElement* element, float x);

void fElementSetTopLayer(fElement* element);
void fElementSetBottomLayer(fElement* element);
void fElementMoveLayerDown(fElement* element);
void fElementMoveLayerUp(fElement* element);

float fElementGetWidth (fElement* element);
void  fElementSetWidth (fElement* element, float size);

float fElementGetHeight(fElement* element);
void  fElementSetHeight(fElement* element, float size);

void fElementSetFont(fElement* element, sTexture* font);
void fElementSetPlaneColor4fv(fElement* element, float* color);
void fElementGetPlaneColor4fv(fElement* element, float* color);
void fElementSetTextColor4fv(fElement* element, float* color);
void fElementGetTextColor4fv(fElement* element, float* color);

#endif /* SRC_FTEXT_H_ */
