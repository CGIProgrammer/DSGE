/*
 * forms.c
 *
 *  Created on: 10 авг. 2019 г.
 *      Author: ivan
 */

#include "2D_renderer/2D_renderer.h"
#include "2D_renderer/forms.h"

static int font = 8;
static float button_color[] = {0.5,0.5,0.5,1.0};
static float button_color_hover[] = {0.3,0.3,0.3,1.0};
static float button_text_color[] = {0.75,0.6,0.0,1.0};
static float button_text_color_hover[] = {0.8,0.75,0.2,1.0};

static void _button_hover(fForm* button)
{
	fElement* plane = button->elements[0];
	fElement* text = button->elements[1];
	memcpy(plane->bg_color, button_color_hover, sizeof(button_color_hover));
	memcpy(text->text_color, button_text_color_hover, sizeof(button_text_color_hover));
}

static void _button_unhover(fForm* button)
{
	fElement* plane = button->elements[0];
	fElement* text = button->elements[1];
	memcpy(plane->bg_color, button_color, sizeof(button_color));
	memcpy(text->text_color, button_text_color, sizeof(button_text_color));
}

void fButtonConstructor(fButton* button, char* text, int x, int y, int width, int height, void(*callback)(fForm*))
{
	if (height==-1) height = font*4*2/3;
	if (width ==-1)
	{
		if (text)
			width = (strlen(text)+2)*font;
		else
			width = 10.0;
	}
	fFormConstructor(button);
	button->lock_rotation = 1;
	button->width = width;
	button->height = height;
	fElement* background = fFormAddElement(button, 0, font, width, height);
	background->lock_rotation = 1;
	memcpy(background->bg_color, button_color, sizeof(button_color));

	fElement* label = fFormAddElement(button, 0, font, width, height);
	label->lock_rotation = 1;
	label->align = 0;
	fElementSetText(label, text);
	memcpy(label->text_color, button_text_color, sizeof(button_text_color));
	fElementSetTopLayer(label);
	fElementTranslateLocal(label, 0.0, height/5);

	fFormSetCursorHover(button, _button_hover);
	fFormSetCursorLeave(button, _button_unhover);
	fFormSetLMB(button, callback);

	fFormTranslateGlobal(button, x, y);
}

fButton* fButtonCreate(char* text, int x, int y, int width, int height, void(*callback)(fForm*))
{
	fForm* button = sMalloc(sizeof(fForm));
	fButtonConstructor(button, text, x, y, width, height, callback);
	button->ID = -button->ID;
	return (fButton*)button;
}

void fButtonSetCallback(fButton* bttn, void(*callback)(fForm*))
{
	fFormSetLMB(bttn, callback);
}

int fButtonGetTextLength(fButton* bttn)
{
	return ((fElement*)bttn->elements[1])->length;
}

void fButtonSetText(fButton* bttn, char* text)
{
	fElement* txt = fFormGetElement(bttn, 1);
	fElementSetText(txt, text);
}

void fButtonGetText(fButton* bttn, char* text, int buff_size)
{
	char* bttn_text = ((fElement*)bttn->elements[1])->text;
	
	if (buff_size>-1)
		memcpy(text, bttn_text, MIN(strlen(bttn_text), buff_size));
	else
		memcpy(text, bttn_text, strlen(bttn_text));
}

void fButtonDelete(fButton* bttn)
{
	fFormDelete(bttn);
}

static void _list_callback(fForm* form, int scroll)
{
	fForm* el = form->children[form->child_count-1];
	float height = el->transform[5] - form->height;
	fFormScrollVertical(form, -scroll*form->height*0.25);
	if (form->yscroll>height) form->yscroll = height;
	if (form->yscroll<0) form->yscroll = 0;
	fFormScrollVertical(form, 0);
}

static void _list_lmb_callback(fForm* form)
{
	fForm* list = form->parent;
	int index = listIndexOf(list->children, list->child_count, form);
	void(*func)(fForm*, int) = ((fForm*)form->parent)->data;
	if (func)
	{
		func(form, index);
	}
}

void fListConstructor(fList* list, float x, float y, float width, float height, void(*callback)(fForm*,int))
{
	fFormConstructor(list);
	list->data = callback;
	list->width = width;
	list->height = height;

	fElement* bg = fFormAddElement(list, 0, font, width, height);
	fFormTranslateGlobal(list, x, y);
	bg->bg_color[3] = 1.0;
	
	fForm* bttn = fButtonCreate(0, x+1, y+1, width-2, -1, 0);
	fFormAddForm(list, bttn);
	fFormSetLMB(bttn, _list_lmb_callback);
	y += bttn->height + 1;
	
	fForm* end = fFormCreate();
	fFormTranslateLocal(end, x, y);
	fFormAddForm(list, end);
	fFormSetScroll(list, _list_callback);
}

fList* fListCreate(float x, float y, float width, float height, void(*callback)(fForm*,int))
{
	fForm* result = sMalloc(sizeof(fList));
	fListConstructor(result, x, y, width, height, callback);
	result->ID = -result->ID;
	return (fList*)result;
}

void fListAddItem(fList* list, char* text)
{
	fForm* endElement = list->children[list->child_count-1];
	fForm* lastElement = list->children[list->child_count-2];
	float width = lastElement->width;
	float height = lastElement->height;
	void(*callback)(fForm*) = (void(*)(fForm*))lastElement->on_left_click;
	if (!fButtonGetTextLength(lastElement))
	{
		fListRemoveItem(list, -2);
	}

	fForm* item = fButtonCreate(text, 0,0, width, height, callback);
	fFormAddForm(list, item);
	fFormSetLocalPosition(item, endElement->transform[2]+1, endElement->transform[5]+1);
	fFormTranslateLocal(endElement, 0, lastElement->height + 1);
	fFormSetTopLayer(endElement);
}

void fListRemoveItem(fList* list, int index)
{
	if (index<0) index += list->child_count;

	fForm* item = list->children[index];
	
	for (int i=index; i<list->child_count; i++)
	{
		fFormTranslateLocal(list->children[i], 0.0, -(item->height+1));
	}
	fFormDelete(item);
}

void fListDelete(fList* form)
{
	fFormDelete(form);
}
