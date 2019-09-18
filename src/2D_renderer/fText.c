/*
 * fText.c
 *
 *  Created on: 6 авг. 2019 г.
 *      Author: ivan
 */

#include "2D_renderer/2D_renderer.h"

extern uint32_t activeShader;

extern fElement** elements;
extern int elements_count;

fElement* fElementCreate(char* text, int font_size, int width, int height, int carry)
{
	fElement* ft = sCalloc(sizeof(fElement),1);
	memset(ft, 0, sizeof(fElement));
	listPushBack((void**)&elements, &elements_count, &ft, sizeof(ft));
	ft->ID = ID_counter++;
	ft->dynamic_carry = carry;
	ft->font_width = font_size;
	ft->cursor1 = -1;
	ft->cursor2 = -1;
	ft->width = width;
	ft->height = height;
	ft->font = &default_font;
	ft->align = -1;
	ft->uv_size[0] = 1;
	ft->uv_size[1] = 1;
	if (text)
	{
		fElementSetText(ft, text);
	}

	for (int i=0; i<9; i++)
	{
		ft->transform_global[i] = ft->transform[i] = i%4 == 0;
	}
	return ft;
}

void  fElementSetVisibleBit(fElement* element, _Bool bit)
{
	element->visible = bit;
}

_Bool fElementGetVisibleBit(fElement* element)
{
	return element->visible;
}

void fElementSetLockRotationBit(fElement* element, _Bool bit)
{
	element->lock_rotation = bit;
}

_Bool fElementGetLockRotationBit(fElement* element)
{
	return element->lock_rotation;
}

static void _setTextFrame(uint16_t width, int16_t height, int32_t scrolling)
{
	text_width = width;
	text_htight = height;
	text_scrolling = scrolling;
}

static void _drawCharacter(uint8_t chr, float x, float y)
{
	float pos[] = {x, y};
	glc(sMaterialUniformi(activeShader, "char_index", chr));
	glc(sMaterialUniformfv(activeShader, "char_position", pos, 2));
	glc(glDrawElements(GL_TRIANGLES, 6, 0x1401+sizeof(index_t),BUFFER_OFFSET(0)));
}

void fElementSetFont(fElement* element, sTexture* font)
{
	element->font = font;
}

void fElementSetPlaneColor4f(fElement* element, float r, float g, float b, float a)
{
	element->bg_color[0] = r;
	element->bg_color[1] = g;
	element->bg_color[2] = b;
	element->bg_color[3] = a;
}

void fElementSetPlaneColor4fv(fElement* element, float* color)
{
	memcpy(element->bg_color, color, sizeof(float[4]));
}

void fElementGetPlaneColor4fv(fElement* element, float* color)
{
	memcpy(color, element->bg_color, sizeof(float[4]));
}

void fElementSetTextColor4fv(fElement* element, float* color)
{
	memcpy(element->text_color, color, sizeof(float[4]));
}

void fElementGetTextColor4fv(fElement* element, float* color)
{
	memcpy(color, element->text_color, sizeof(float[4]));
}

static uint32_t fElementGetLineLength(fElement* ft, uint32_t line)
{
	int result = 0;
	if (line >= ft->lines_count) return -1;
	for (char* chr=ft->lines[line]; chr < ft->lines[line+1]; chr++)
	{
		result += *(uint8_t*)chr >= 32;
	}
	return result;
}

static void fElementCountLines(fElement* text)
{
	uint16_t chars_width = roundf((float)text->width / text->font_width);
	char* ptr;
	
    text->lines_count = 0;
    ptr = text->text;
	listPushBack((void**)&text->lines, &text->lines_count, &ptr, sizeof(ptr));

	if (text->dynamic_carry)
	{
		int line_start_ind = 0;
		int lastWordIndex = 0;
		for (int i=1; i<text->length; i++)
		{
			if (text->text[i]==' ')
			{
				lastWordIndex = i+1;
			}
			if (i - line_start_ind >= chars_width || text->text[i] == '\n')
			{
				if (!lastWordIndex || text->text[i] == '\n')
				{
					line_start_ind = i+1;
				}
				else
                {
                    line_start_ind = lastWordIndex;
                }
				ptr = text->text + line_start_ind;
				listPushBack((void**)&text->lines, &text->lines_count, &ptr, sizeof(ptr));
				lastWordIndex = 0;
			}
		}
	}
	else
	{
		for (int i=1; i<text->length; i++)
		{
			if (text->text[i] == '\n')
			{
				ptr = text->text + i+1;
				listPushBack((void**)&text->lines, &text->lines_count, &ptr, sizeof(ptr));
			}
		}
	}
	ptr = text->text + text->length;
	listPushBack((void**)&text->lines, &text->lines_count, &ptr, sizeof(ptr));
	text->lines_count--;
}

void fElementApplyTransformToGlobal(fElement* ft)
{
	fForm* form = ft->form;
	if (form)
	{
		if (ft->lock_rotation)
		{
			ft->transform_global[2] = ft->transform[2] + (form->transform_global[2]/* - form->xscroll*/);
			ft->transform_global[5] = ft->transform[5] + (form->transform_global[5]/* - form->yscroll*/);
		}
		else
		{
			float m[9];
			memcpy(m, form->transform_global, sizeof(float[9]));
			//m[2] -= form->xscroll;
			//m[5] -= form->yscroll;
			MulArrays3x3(ft->transform_global, ft->transform, m);
		}
	}
	else
	{
		memcpy(ft->transform_global, ft->transform, sizeof(float[9]));
	}
}

void fElementApplyTransformToLocal(fElement* element)
{
	if (element->form)
	{
		fForm* form = element->form;
		if (element->lock_rotation)
		{
			element->transform[2] = element->transform_global[2] - (form->transform_global[2] - form->xscroll);
			element->transform[5] = element->transform_global[5] - (form->transform_global[5] - form->yscroll);
		}
		else
		{
			float m[9];
			memcpy(m, form->transform_global, sizeof(float[9]));
			m[2] -= form->xscroll;
			m[5] -= form->yscroll;
			InvertArray3x3(m, m);
			MulArrays3x3(element->transform, element->transform_global, m);
		}
	}
	else
	{
		memcpy(element->transform, element->transform_global, sizeof(element->transform_global));
	}
}

void fElementTranslateGlobal(fElement* element, float x, float y)
{
	element->transform_global[2] += x;
	element->transform_global[5] += y;

	fElementApplyTransformToLocal(element);
}

void fElementTranslateLocal(fElement* form, float x, float y)
{
	form->transform[2] += form->transform[0] * x;
	form->transform[5] += form->transform[3] * x;
	form->transform[2] += form->transform[1] * y;
	form->transform[5] += form->transform[4] * y;
	fElementApplyTransformToGlobal(form);
}

void fElementSetLocalPosition(fElement* element, float x, float y)
{
	element->transform[2] = x;
	element->transform[5] = y;
	fElementApplyTransformToGlobal(element);
}
void fElementSetGlobalPosition(fElement* element, float x, float y)
{
	element->transform_global[2] = x;
	element->transform_global[5] = y;
	fElementApplyTransformToLocal(element);
}

void fElementGetLocalPosition(fElement* element, float* x, float* y)
{
	*x = element->transform[2];
	*y = element->transform[5];
}
void fElementGetGlobalPosition(fElement* element, float* x, float* y)
{
	*x = element->transform_global[2];
	*y = element->transform_global[5];
}

float fElementGetLocalRotation(fElement* element)
{
	return atan2f(element->transform[3], element->transform[0]);
}

void  fElementSetLocalRotation(fElement* element, float x)
{
	if (!element->lock_rotation)
	{
		float rotation[9];
		RotationArray3x3(rotation, x);
		memcpy(element->transform + 0, rotation + 0, sizeof(float[2]));
		memcpy(element->transform + 3, rotation + 3, sizeof(float[2]));
		fElementApplyTransformToGlobal(element);
	}
}

float fElementGetGlobalRotation(fElement* element)
{
	return atan2f(element->transform_global[3], element->transform_global[0]);
}

void  fElementSetGlobalRotation(fElement* element, float x)
{
	if (!element->lock_rotation)
	{
		float rotation[9];
		RotationArray3x3(rotation, x);
		memcpy(element->transform_global + 0, rotation + 0, sizeof(float[2]));
		memcpy(element->transform_global + 3, rotation + 3, sizeof(float[2]));
		fElementApplyTransformToLocal(element);
	}
}

void fElementRotate(fElement* element, float x)
{
	if (!element->lock_rotation)
	{
		float rotation[9];
		RotationArray3x3(rotation, x);
		MulArrays3x3(element->transform, element->transform, rotation);
		fElementApplyTransformToGlobal(element);
	}
}

void fElementSetText(fElement* ft, char* text)
{
	if (!text)
	{
		ft->length = 0;
		ft->lines_count = 0;
		sFree(ft->text);
		sFree(ft->lines);
		return;
	}
	ft->length = strlen(text);
	ft->text = sRealloc(ft->text, strlen(text)+1);
	strcpy(ft->text, text);
	ft->text[ft->length] = 0;
    fElementCountLines(ft);
    if (ft->text_color[3]==0.0)
    {
    	ft->text_color[3] = 1.0;
    }
}

int  fElementGetTextLength(fElement* element)
{
	return element->length;
}

void fElementGetText(fElement* element, char* text, int length)
{
	if (length<0 || length>element->length)
	{
		length = element->length + 1;
	}
	memcpy(text, element->text, length);
}

char* fElementGetTextPtr(fElement* element)
{
	return element->text;
}

void fElementSetImage(fElement* element, sTexture* image)
{
	element->bg_image = image;
}

sTexture* fElementGetImage(fElement* element)
{
	return element->bg_image;
}

void fElementSetImageCoords(fElement* element, float* coords)
{
	element->uv_start[0] = coords[0];
	element->uv_start[1] = coords[1];
}

void fElementGetImageCoords(fElement* element, float* coords)
{
	coords[0] = element->uv_start[0];
	coords[1] = element->uv_start[1];
}

void fElementSetImageSize(fElement* element, float* size)
{
	element->uv_size[0] = size[0];
	element->uv_size[1] = size[1];
}

void fElementGetImageSize(fElement* element, float* size)
{
	size[0] = element->uv_size[0];
	size[1] = element->uv_size[1];
}

void fElementDrawRect(fElement* element)
{
	if (element->bg_color[3]==0) return;
	useProgram(_rectangle_shader.program);
	glc(glBindBuffer(GL_ARRAY_BUFFER, _rectangle.VBO));
	glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, _rectangle.IBO));

	glc(glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)0));
	glc(glEnableVertexAttribArray(0));
	glc(glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)12));
	glc(glEnableVertexAttribArray(1));
	glc(glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)24));
	glc(glEnableVertexAttribArray(1));
	glc(glVertexAttribPointer(3, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)32));
	glc(glEnableVertexAttribArray(3));
	glc(glVertexAttribPointer(4, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)44));
	glc(glEnableVertexAttribArray(4));

	int x = sEngineGetWidth();
	int y = sEngineGetHeight();

	glc(sMaterialUniformf(activeShader,"rect_width", element->width));
	glc(sMaterialUniformf(activeShader,"rect_height", element->height));
	glc(sMaterialUniformf(activeShader,"width", x));
	glc(sMaterialUniformf(activeShader,"height", y));
	glc(sMaterialUniformfv(activeShader,"color", element->bg_color, 4));
	glc(sMaterialUniformfv(activeShader,"transform", element->transform_global, 9));
	glc(sMaterialUniformfv(activeShader,"uv_start", element->uv_start, 2));
	glc(sMaterialUniformfv(activeShader,"uv_size", element->uv_size, 2));
	if (element->bg_image) {
		glc(sMaterialUniformi(activeShader,"use_texture", 1));
		glc(sMaterialTexture(activeShader, "background", element->bg_image->ID, 3));
	}
	else {
		glc(sMaterialUniformi(activeShader,"use_texture", 0));
		glc(sMaterialTexture(activeShader, "background", 0, 3));
	}

	glc(glDrawElements(GL_TRIANGLES, 6, 0x1401+sizeof(index_t),BUFFER_OFFSET(0)));
}

void fElementDrawText(fElement* ft)
{
	if (!ft->text) return;
	useProgram(_font_shader.program);
	_setTextFrame(ft->width, ft->height, 0);

	glc(glBindBuffer(GL_ARRAY_BUFFER, _rectangle.VBO));
	glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, _rectangle.IBO));
	
	glc(glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)0));
	glc(glEnableVertexAttribArray(0));
	glc(glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)12));
	glc(glEnableVertexAttribArray(1));
	glc(glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)24));
	glc(glEnableVertexAttribArray(1));
	glc(glVertexAttribPointer(3, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)32));
	glc(glEnableVertexAttribArray(3));
	glc(glVertexAttribPointer(4, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)44));
	glc(glEnableVertexAttribArray(4));

	glc(sMaterialUniformf(activeShader,"char_width", ft->font_width));
	glc(sMaterialUniformf(activeShader,"char_height", ft->font_width * ft->font->height / ft->font->width));
	glc(sMaterialUniformf(activeShader,"width", sEngineGetWidth()));
	glc(sMaterialUniformf(activeShader,"height",sEngineGetHeight()));
	glc(sMaterialUniformfv(activeShader,"color", ft->text_color, 4));
	glc(sMaterialTexture(activeShader, "font", ft->font->ID, 3));
	glc(sMaterialUniformfv(activeShader,"transform", ft->transform_global, 9));

	int char_height = ft->font_width * ft->font->height / ft->font->width;
	int char_width = ft->font_width;

	float dy = 0;
	for (int line=0; line < ft->lines_count; line++)
	{
		float dx;
		if (ft->align < 0)
		{
			dx = 0;
			for (char* chr=ft->lines[line]; chr<ft->lines[line+1]; chr++)
			{
				if (*(uint8_t*)chr >= 32)
				{
					_drawCharacter(*chr, dx, dy);
					dx += char_width;
				}
			}
		}
		if (ft->align == 0)
		{
			dx = (ft->width - fElementGetLineLength(ft, line)*char_width)/2;
			for (char* chr=ft->lines[line]; chr<ft->lines[line+1]; chr++)
			{
				if (*(uint8_t*)chr >= 32)
				{
					_drawCharacter(*chr, dx, dy);
					dx += char_width;
				}
			}
		}
		if (ft->align > 0)
		{
			dx = ft->width - char_width;
			char* start = ft->lines[line+1];
			char* end = ft->lines[line];
			for (char* chr=start-1 - (*start!=0); chr >= end; chr--)
			{
				if (*(uint8_t*)chr >= 32)
				{
					_drawCharacter(*chr, dx, dy);
					dx -= char_width;
				}
			}
		}
		dy+= char_height;
	}
}

void fElementDelete(fElement* ft)
{
	sFree(ft->lines);
	sFree(ft->text);
	fForm* form = ft->form;
	listPopPointer((void**)&elements, &elements_count, ft);
	if (form)
	{
		listPopPointer((void**)&form->elements, &form->elements_count, ft);
	}
	sFree(ft);
}

void fFormSetTopLayer(fForm* form)
{
	fForm* parent = form->parent;
	if (parent)
	{
		int index1 = listIndexOf((void**)parent->children, parent->child_count, form);
		int index2 = parent->child_count - 1;
		if (index1>-1 && index2>-1 && index1<index2)
		{
			listSwapElements((void**)parent->children, index1, index2);
		}
	}
}

void fElementSetTopLayer(fElement* element)
{
	fForm* form = element->form;
	if (form)
	{
		int index1 = listIndexOf((void**)form->elements, form->elements_count, element);
		int index2 = form->elements_count - 1;
		if (index1>-1 && index2>-1 && index1<index2)
		{
			listSwapElements((void**)form->elements, index1, index2);
		}
	}
}

void fElementSetBottomLayer(fElement* element)
{
	fForm* form = element->form;
	if (form)
	{
		int index1 = listIndexOf((void**)form->elements, form->elements_count, element);
		int index2 = 0;
		if (index1>-1)
		{
			listSwapElements((void**)form->elements, index1, index2);
		}
	}
}
void fElementMoveLayerDown(fElement* element)
{
	fForm* form = element->form;
	if (form)
	{
		int index1 = listIndexOf((void**)form->elements, form->elements_count, element);
		int index2 = index1 - 1;
		if (index1>-1 && index2>-1)
		{
			listSwapElements((void**)form->elements, index1, index2);
		}
	}
}
void fElementMoveLayerUp(fElement* element)
{
	fForm* form = element->form;
	if (form)
	{
		int index1 = listIndexOf((void**)form->elements, form->elements_count, element);
		int index2 = index1 + 1;
		if (index1>-1 && index1<form->elements_count && index2<form->elements_count)
		{
			listSwapElements((void**)form->elements, index1, index2);
		}
	}
}

float fElementGetWidth (fElement* element)
{
	return element->width;
}

void  fElementSetWidth (fElement* element, float size)
{
	if (size>0)
		element->width = size;
}

float fElementGetHeight(fElement* element)
{
	return element->height;
}

void  fElementSetHeight(fElement* element, float size)
{
	if (size>0)
		element->height = size;
}

void fElementPrint(fElement* ft)
{
	int i = 0;
	for (char* txt = ft->text; *txt; txt++)
	{
		if (*txt < 32) continue;
		putchar(*txt);
		if (ft->lines[i] == txt)
		{
            if (i)
                putchar('\n');
			i++;
		}
	}
    putchar('\n');
}

int fElementCheckHover(fElement* element, float x, float y)
{
	int result = 0;
	float cursor[3] = {x,y,1};
	cursor[0] -= element->transform_global[2];
	cursor[1] -= element->transform_global[5];

	if (!element->lock_rotation)
	{
		MulVectorByMatrixArray3x3(cursor,cursor,element->transform_global);
	}

	result = cursor[0]>=0 && cursor[1]>=0 && cursor[0]<=element->width && cursor[1]<=element->height;
	if (element->form)
	{
		result &= fFormCheckHoverBounds(element->form, x, y);
	}

	if (element->hovered==0 && result)
	{
		element->hovered = 1;
	}
	else if (element->hovered==1 && result)
	{
		element->hovered = 2;
	}
	else if (element->hovered==2 && !result)
	{
		element->hovered = 3;
	}
	else if (element->hovered==3 && !result)
	{
		element->hovered = 0;
	}

	return element->hovered;
}
