/*
 * forms.h
 *
 *  Created on: 13 авг. 2019 г.
 *      Author: ivan
 */

#ifndef INC_2D_RENDERER_FORMS_H_
#define INC_2D_RENDERER_FORMS_H_

typedef fForm fButton;
typedef fForm fList;

void fButtonConstructor(fButton* button, char* text, int x, int y, int width, int height, void(*callback)(fForm*));
fButton* fButtonCreate(char* text, int x, int y, int width, int height, void(*callback)(fForm*));
int  fButtonGetTextLength(fButton* bttn);
void fButtonSetCallback(fButton* bttn, void(*callback)(fForm*));
void fButtonGetText(fButton* bttn, char* text, int buff_size);
void fButtonSetText(fButton* bttn, char* text);
void fButtonDelete(fButton* bttn);

void fListConstructor(fList* list, float x, float y, float width, float height, void(*callback)(fForm*,int));
fList* fListCreate(float x, float y, float width, float height, void(*callback)(fForm*,int));
void fListAddItem(fList*, char*);
void fListRemoveItem(fList* list, int index);
void fListDelete(fList* form);
#endif /* INC_2D_RENDERER_FORMS_H_ */
