/*
 * fForm.c
 *
 *  Created on: 6 авг. 2019 г.
 *      Author: ivan
 */

#include "2D_renderer/fForm.h"

extern fForm**  forms;
extern int forms_count;
static void(*garbage_collector)(void) = 0;
static int _postfunc_interval = 0, _postfunc_interval_counter = 0;

//void sMaterialUniformi(GLuint, char*, long);
void sMaterialUniformf(GLuint, char*, float);

static void addForm(fForm* form)
{
	int i=0;
	for (i=0; i<forms_count && forms[i]; i++);
	if (i==forms_count)
	{
		listPushBack((void**)&forms, &forms_count, &form, sizeof(&form));
	}
	else
	{
		forms[i] = form;
	}
}

fForm* fFormCreate(void)
{
	fForm* form = sMalloc(sizeof(fForm));
	fFormConstructor(form);
	form->ID = -form->ID;
	return form;
}

void fFormConstructor(fForm* form)
{
	memset(form, 0, sizeof(fForm));
	form->ID = ID_counter++;
	form->lock_rotation = 1;
	form->visible = 1;
	form->limits = 1;
	IdentityArray3x3(form->transform);
	IdentityArray3x3(form->transform_global);
	addForm(form);
}

fElement* fFormGetElement(fForm* form, int index)
{
	if (index<0)
	{
		index += form->elements_count;
	}
	if (index<0 || index>=form->elements_count)
	{
		return (fElement*)0;
	}
	return form->elements[index];
}

int fFormGetChildCount(fForm* form)
{
	if (form) return form->child_count;
	else return 0;
}

fForm* fFormGetChild(fForm* form, int index)
{
	if (!form) return 0;
	if (index<0)
	{
		index += form->child_count;
	}
	if (index<0 || index>=form->child_count)
	{
		return (fForm*)0;
	}
	return form->children[index];
}

void fFormAddForm(fForm* form, fForm* child)
{
	fForm* stack[128];
	int i=0;
	for (fForm* parent = form->parent;parent;parent = parent->parent)
	{
		for (int j=0;j<i;j++)
		{
			if (stack[j]==parent)
			{
				fputs("Parent looping detected\n", stderr);
				for (int k=0;k<i;k++)
				{
					fprintf(stderr, "%d -> ", stack[k]->ID);
				}
				fprintf(stderr, "%d\n", parent->ID);
				exit(-1);
			}
		}
		stack[i] = parent;
		i++;
	}
	fFormRemoveParent(child);
	listPushBack((void**)&form->children, &form->child_count, &child, sizeof(&child));
	child->parent = form;
	fFormApplyTransformToLocal(child);
}

void fFormRemoveParent(fForm* gp)
{
	fForm* parent = gp->parent;
	if (parent)
	{
		listPopPointer((void**)&parent->children, &parent->child_count, gp);
		gp->parent = 0;
	}
}

void fFormMarkDelete(fForm* form)
{
	form->delete_me = 1;
}

void fFormDelete(fForm* form)
{
	while (form->elements_count)
	{
		fElementDelete(form->elements[0]);
	}
	while (form->child_count)
	{
		fFormDelete(form->children[0]);
	}

	fFormRemoveParent(form);

	listPopPointer((void**)&forms, &forms_count, form);

	sFree(form->elements);
	sFree(form->children);
	if (form->ID<0)
		sFree(form);
}

fElement* fFormAddElement(fForm* form, char* text, int font_size, float width, float height)
{
	fElement* ft = fElementCreate(text, font_size, width, height, 1);
	listPushBack((void**)&form->elements, &form->elements_count, &ft, sizeof(&ft));
	ft->form = form;
	return ft;
}

/*void fFormSortByLayers(fForm* form)
{
	int _compare(void* a, void* b)
	{
		fForm* form_a = a;
		fForm* form_b = b;
		if (form_a->layer > form_b->layer)
		{
			return 1;
		}
		if (form_a->layer > form_b->layer)
		{
			return -1;
		}
		return 0;
	}
	qsort(form->elements, form->elements_count, sizeof(fElement), (int(*)(const void*,const void*))_compare);
}
*/

void fFormDrawBounds(fForm* form)
{
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

	float color[4] = {0.0,0.0,0.0,0.0};

	glc(sMaterialUniformf(activeShader,"rect_width", form->width));
	glc(sMaterialUniformf(activeShader,"rect_height", form->height));
	glc(sMaterialUniformf(activeShader,"width", x));
	glc(sMaterialUniformf(activeShader,"height", y));
	glc(sMaterialUniformfv(activeShader,"color", color, 4));
	glc(sMaterialUniformfv(activeShader,"transform", form->transform_global, 9));
	glc(sMaterialUniformi(activeShader,"use_texture", 0));
	glc(sShaderValidate());
	glc(glDrawElements(GL_TRIANGLES, 6, 0x1401+sizeof(index_t),BUFFER_OFFSET(0)));
}

void fFormDraw(fForm* form, int z)
{
	_Bool limits = form->limits;

	glc(glEnable(GL_STENCIL_TEST));
	glc(glStencilMask(0xFF));

	if (limits)
	{
		glc(glStencilFunc(GL_EQUAL, z, 0xFF));
		glc(glStencilOp(GL_KEEP, GL_KEEP, GL_INCR));
		fFormDrawBounds(form);
	}

	glc(glStencilFunc(GL_EQUAL, z+limits, 0xFF));

	glc(glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP));

	for (int i=0; i<form->elements_count; i++)
	{
		fElementDrawRect(form->elements[i]);
		fElementDrawText(form->elements[i]);
	}
	for (int i=0;i<form->child_count; i++)
	{
		fForm* child = form->children[i];
		if (limits)
		{
			if (child->transform[2]-form->xscroll > form->width ||
				child->transform[5]-form->yscroll > form->height ||
				child->transform[2]-form->xscroll+child->width < 0 ||
				child->transform[5]-form->yscroll+child->width < 0)
				{
					continue;
				}
		}
		fFormDraw(child, z+limits);
	}
	if (limits)
	{
		glc(glStencilFunc(GL_LEQUAL, z, 0xFF));
		glc(glStencilOp(GL_KEEP, GL_KEEP, GL_REPLACE));
		fFormDrawBounds(form);
	}
}

void fFormSetIdle(fForm* form, void(*callback)(fForm*)) {form->idle = (void(*)(void*))callback;}
void fFormSetLMB(fForm* form, void(*callback)(fForm*)) {form->on_left_click = (void(*)(void*))callback;}
void fFormSetRMB(fForm* form, void(*callback)(fForm*)) {form->on_right_click = (void(*)(void*))callback;}
void fFormSetScroll(fForm* form, void(*callback)(fForm*,int)) {form->on_scroll = (void(*)(void*,int))callback;}
void fFormSetCursorHover(fForm* form, void(*callback)(fForm*)) {form->on_pointing = (void(*)(void*))callback;}
void fFormSetCursorLeave(fForm* form, void(*callback)(fForm*)) {form->on_release = (void(*)(void*))callback;}

void fFormSetVisibleBit(fForm* form, _Bool bit) {form->visible = bit;}
void fFormSetLimitsBit(fForm* form, _Bool bit) {form->limits = bit;}
void fFormSetXRayBit(fForm* form, _Bool bit) {form->xray = bit;}
void fFormSetGhostBit(fForm* form, _Bool bit) {form->ghost = bit;}

_Bool fFormGetVisibleBit(fForm* form) {return form->visible;}
_Bool fFormGetLimitsBit(fForm* form) {return form->limits;}
_Bool fFormGetXRayBit(fForm* form) {return form->xray;}
_Bool fFormGetGhostBit(fForm* form) {return form->ghost;}

void fFormSetWidth(fForm* form, float w)
{
	if (w<0) w = 0;
	form->width = w;
};

void fFormSetHeight(fForm* form, float h)
{
	if (h<0) h = 0;
	form->height = h;
};

float  fFormGetWidth(fForm* form)  {return form->width;};
float  fFormGetHeight(fForm* form) {return form->height;};

void fFormApplyTransformToLocal(fForm* form)
{
	fForm* parent = form->parent;
	if (form->parent)
	{
		if (form->lock_rotation)
		{
			form->transform[2] = form->transform_global[2] - (parent->transform_global[2] - parent->xscroll);
			form->transform[5] = form->transform_global[5] - (parent->transform_global[5] - parent->yscroll);
		}
		else
		{
			float group_tr[9];
			float group_tr_inv[9];
			memcpy(group_tr, parent->transform_global, sizeof(group_tr));
			group_tr[2] -= parent->xscroll;
			group_tr[5] -= parent->yscroll;
			InvertArray3x3(group_tr_inv, group_tr);
			MulArrays3x3(form->transform, form->transform_global, group_tr_inv);
		}
	}
	else
	{
		memcpy(form->transform, form->transform_global, sizeof(float[9]));
	}
}

void fFormApplyTransformToGlobal(fForm* form)
{
	fForm* parent = form->parent;

	if (parent)
	{
		if (form->lock_rotation)
		{
			form->transform_global[2] = parent->transform_global[2] + (form->transform[2] - parent->xscroll);
			form->transform_global[5] = parent->transform_global[5] + (form->transform[5] - parent->yscroll);
		}
		else
		{
			float m[9];
			memcpy(m, parent->transform_global, sizeof(float[9]));
			m[2] -= form->xscroll;
			m[5] -= form->yscroll;
			MulArrays3x3(form->transform_global, form->transform, m);
		}
	}
	else
	{
		memcpy(form->transform_global, form->transform, sizeof(float[9]));
	}
}

void fFormApplyTransformToChildren(fForm* form)
{
	for (int i=0; i<form->elements_count; i++)
	{
		fElementApplyTransformToGlobal(form->elements[i]);
	}
	for (int i=0; i<form->child_count; i++)
	{
		fFormApplyTransformToGlobal(form->children[i]);
		fFormApplyTransformToChildren(form->children[i]);
	}
}

void fFormScrollVertical  (fForm* form, float v)
{
	form->yscroll += v;
	fFormApplyTransformToChildren(form);
}

void fFormScrollHorizontal(fForm* form, float h)
{
	form->xscroll += h;
	fFormApplyTransformToChildren(form);
}

void fFormSetVerticalScrolling(fForm* form, float v)
{
	form->yscroll = v;
	fFormApplyTransformToChildren(form);
}

void fFormSetHorizontalScrolling(fForm* form, float h)
{
	form->xscroll = h;
	fFormApplyTransformToChildren(form);
}

float fFormGetVerticalScrolling(fForm* form)
{
	return form->yscroll;
}

float fFormGetHorizontalScrolling(fForm* form)
{
	return form->xscroll;
}

void fFormSetGlobalPosition(fForm* form, float x, float y)
{
	form->transform_global[2] += x;
	form->transform_global[5] += y;

	fFormApplyTransformToLocal(form);
	fFormApplyTransformToChildren(form);
}

void fFormSetLocalPosition(fForm* form, float x, float y)
{
	form->transform[2] = x;
	form->transform[5] = y;
	fFormApplyTransformToGlobal(form);
	fFormApplyTransformToChildren(form);
}

void fFormGetGlobalPosition(fForm* form, float* pos)
{
	if (pos==0) return;
	pos[0] = form->transform_global[2];
	pos[1] = form->transform_global[5];
}

void fFormGetLocalPosition(fForm* form, float* pos)
{
	if (pos==0) return;
	pos[0] = form->transform[2];
	pos[1] = form->transform[5];
}

void fFormTranslateGlobal(fForm* form, float x, float y)
{
	form->transform_global[2] = x;
	form->transform_global[5] = y;

	fFormApplyTransformToLocal(form);
	fFormApplyTransformToChildren(form);
}

void fFormTranslateLocal(fForm* form, float x, float y)
{
	form->transform[2] += form->transform[0] * x;
	form->transform[5] += form->transform[3] * x;
	form->transform[2] += form->transform[1] * y;
	form->transform[5] += form->transform[4] * y;
	fFormApplyTransformToGlobal(form);
	fFormApplyTransformToChildren(form);
}

void fFormSetRotationLocal(fForm* form, float x)
{
	float rotation[9];
	RotationArray3x3(rotation, x);
	memcpy(form->transform + 0, rotation + 0, sizeof(float[2]));
	memcpy(form->transform + 3, rotation + 3, sizeof(float[2]));
	memcpy(form->transform + 6, rotation + 6, sizeof(float[2]));
	fFormApplyTransformToGlobal(form);
	fFormApplyTransformToChildren(form);
}

void fFormSetRotationGlobal(fForm* form, float x)
{
	float rotation[9];
	RotationArray3x3(rotation, x);
	memcpy(form->transform_global + 0, rotation + 0, sizeof(float[2]));
	memcpy(form->transform_global + 3, rotation + 3, sizeof(float[2]));
	memcpy(form->transform_global + 6, rotation + 6, sizeof(float[2]));
	fFormApplyTransformToLocal(form);
	fFormApplyTransformToChildren(form);
}

void fFormRotate(fForm* form, float x)
{
	if (!form->lock_rotation)
	{
		float rotation[9];
		RotationArray3x3(rotation, x);
		MulArrays3x3(form->transform, form->transform, rotation);
		fFormApplyTransformToGlobal(form);
		fFormApplyTransformToChildren(form);
	}
}

float fFormGetLocalRotation(fForm* form)
{
	return RotationFromArray3x3(form->transform);
}

float fFormGetGlobalRotation(fForm* form)
{
	return RotationFromArray3x3(form->transform_global);
}

int fFormCheckHoverBounds(fForm* form, float x, float y)
{
	fForm* parent = form->parent;
	float cursor[3] = {x,y,1};
	int result;
	cursor[0] -= form->transform_global[2];
	cursor[1] -= form->transform_global[5];

	if (!form->lock_rotation)
	{
		MulVectorByMatrixArray3x3(cursor,cursor,form->transform_global);
	}
	result = cursor[0]>=0 && cursor[1]>=0 && cursor[0]<=form->width && cursor[1]<=form->height;
	if (parent)
	{
		if (result)
			result &= fFormCheckHoverBounds(parent, x, y);
		else
			return 0;
	}
	return result;
}

int fFormCheckHover(fForm* form, float x, float y)
{
	int result = 0;
	if (fFormCheckHoverBounds(form, x, y))
	{
		for (int i=0; i<form->elements_count; i++)
		{
			result = result || fElementCheckHover(form->elements[i], x, y);
		}
	}

	return result;
}

void fFormCheckHoverForAll(void)
{
	if (!forms_count) return;
	float x,y;
	sMouseGetPosition(&x,&y);
	fForm* hovered = 0;
	for (int i=forms_count-1; i>=0; i--)
	{
		if (!forms[i]) continue;
		int h;
		if (!forms[i]->ghost && (!hovered || forms[i]->xray) && fFormCheckHover(forms[i], x, y))
		{
			hovered = forms[i];
			h = 1;
		}
		else
		{
			h = 0;
		}

		if (forms[i]->hovered==0 && h)
		{
			forms[i]->hovered = 1;
		}
		else if (forms[i]->hovered==1 && h)
		{
			forms[i]->hovered = 2;
		}
		else if (forms[i]->hovered==1 && !h)
		{
			forms[i]->hovered = 3;
		}
		else if (forms[i]->hovered==2 && !h)
		{
			forms[i]->hovered = 3;
		}
		else if (forms[i]->hovered==3 && !h)
		{
			forms[i]->hovered = 0;
		}
	}
}

void fFormExecuteFunctions(fForm* form)
{
	if (form->idle)
	{
		form->idle((void*)form);
	}
	if (form->hovered==1 && form->on_pointing)
	{
		form->on_pointing((void*)form);
	}
	if (form->hovered==3 && form->on_release)
	{
		form->on_release((void*)form);
	}
	if (form->on_scroll && form->hovered)
	{
		int scrolling = (int)sMouseGetVerticalScroll();
		form->on_scroll((void*)form, scrolling);
	}
	if (form->hovered)
	{
		switch (sMouseGetKeyState(0))
		{
		case 1 : if (form->on_left_click)   form->on_left_click((void*)form); break;
		case 3 : if (form->on_left_release) form->on_left_release((void*)form); break;
		}
		switch (sMouseGetKeyState(1))
		{
		case 1 : if (form->on_right_click)   form->on_right_click((void*)form); break;
		case 3 : if (form->on_right_release) form->on_right_release((void*)form); break;
		}
	}
}

void fFormsSetPostFunctionInterval(int interval)
{
	_postfunc_interval = interval;
}

void fFormsSetPostFunction(void(*callback)(void))
{
	garbage_collector = callback;
}

void fFormsDraw(void)
{
	glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
	glc(glViewport(0,0,sEngineGetWidth(),sEngineGetHeight()));
	glc(glClearColor(0.15,0.15,0.15,0.0));
	glc(glClearStencil(0x00));
	glc(glStencilMask(0xFF));
	glc(glDisable(GL_CULL_FACE));
	glc(glDisable(GL_DEPTH_TEST));
	glc(glEnable(GL_STENCIL_TEST));
	glc(glEnable(GL_BLEND));
	glc(glClear(GL_STENCIL_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));

	float x,y;
	sMouseGetPosition(&x, &y);
	for (int i=0; i<forms_count; i++)
	{
		fFormCheckHover(forms[i], x, y);
	}
	for (int i=0; i<forms_count; i++)
	{
		if (!forms[i]->parent)
		{
			fFormDraw(forms[i], 0);
		}
	}
}

void fFormsProcess(void)
{
	fFormCheckHoverForAll();
	for (int i=0; i<forms_count; i++)
	{
		fFormExecuteFunctions(forms[i]);
	}
	fFormsDraw();
	for (int i=forms_count-1; i>=0; i--)
	{
		if (forms[i]->delete_me)
		{
			fFormDelete(forms[i]);
			if (i>0 && i>forms_count)
			{
				i = forms_count - 1;
			}
		}
	}
	if (garbage_collector)
	{
		if (_postfunc_interval_counter>=_postfunc_interval)
		{
			garbage_collector();
			_postfunc_interval_counter = 0;
		}
		_postfunc_interval_counter++;
	}
}
