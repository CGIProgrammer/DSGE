/*
 * game_object.c
 *
 *  Created on: 09 июля 2020 г.
 *      Author: Ivan G
 */

#include "structures/game_object.h"
#include "structures/list.h"
#include "memmanager.h"

static sGameObjectID* objects = 0;

static bool check_parent_loop(sGameObjectID obj1, sGameObjectID obj2)
{
    /*for (sGameObjectID par=obj1; par->parent; par=par->parent)
    {
        if (par==obj2) return 1;
    }
    for (sGameObjectID par=obj2; par->parent; par=par->parent)
    {
        if (par==obj1) return 1;
    }*/
    return 0;
}

sGameObjectID sGameObjectCreate(char* name)
{
    sGameObjectID object = sNew(sGameObject);
    strcpy(object->name, name);
    object->transform.local = laIdentity;
    object->transform.global = laIdentity;
    object->transform.global_prev_1 = laIdentity;
    object->transform.global_prev_2 = laIdentity;
    sListPushBack(objects, object);
    return object;
}

void sGameObjectSetVisual(sGameObjectID obj, sMeshID visual)
{
    if (!obj) return;
    if (obj->visual_component) {
        sListPopItem(obj->visual_component->users, obj);
    }
    if (visual) {
        sListPushBack(visual->users, obj);
    }
    obj->visual_component = visual;
}

void sGameObjectEnd(sGameObjectID obj)
{
    if (obj) obj->ended = 1;
}

void sGameObjectDelete(sGameObjectID obj)
{
    if (!obj) return;
    size_t children_count = sListGetSize(obj->children);
    sListPopItem(objects, obj);
    if (obj->scene)
    {
        sListPopItem(obj->scene->objects, obj);
        sListPopItem(obj->scene->lights,  obj);
    }
    if (obj->visual_component) {
        sGameObjectSetVisual(obj, 0);
    }
    if (obj->camera_component) {
        sCameraComponentDelete(obj->camera_component);
    }
    if (obj->light_component) {
        sLightComponentDelete(obj->light_component);
    }
    sGameObjectRemoveParent(obj);
    for (size_t i=0; i<children_count; i++)
    {
        sGameObjectRemoveParent(obj->children[0]);
    }
    sDelete(obj->behaviour);
    sListPopItem(objects, obj);
    sDelete(obj);
}

void sObjectAddCallback(sGameObjectID obj, sGameObjectCallback callback)
{
    sListPushBack(obj->behaviour, callback);
}

void sGameObjectRemoveParent(sGameObjectID obj)
{
    if (!obj->parent) return;
    sListPopItem(obj->parent->children, obj);
    obj->parent = 0;
}

void sGameObjectAddChild(sGameObjectID obj, sGameObjectID child)
{
    if (obj && child && obj!=child && !check_parent_loop(obj, child))
    {
        sGameObjectRemoveParent(child);
        sListPushBack(obj->children, child);
        child->parent = obj;
    }
}

void sGameObjectSetParent(sGameObjectID obj, sGameObjectID parent)
{
    sGameObjectAddChild(parent, obj);
}

void sGameObjectHide(sGameObjectID obj)
{
    obj->hidden = 1;
}

void sGameObjectShow(sGameObjectID obj)
{
    obj->hidden = 0;
}

void sGameObjectSetVisibility(sGameObjectID obj, bool visible)
{
    obj->hidden = visible;
}

void sGameObjectApplyChildrenTransform(sGameObjectID obj)
{
    if (!obj->parent)
    {
        obj->transform.global = obj->transform.local;
    }
    size_t children_count = sListGetSize(obj->children);
    for (size_t i=0; i<children_count; i++)
    {
        obj->children[i]->transform.global = laMul(obj->children[i]->transform.local, obj->transform.global);
        sGameObjectApplyChildrenTransform(obj->children[i]);
    }
}

void sGameObjectClear(void)
{
    size_t obj_count = sListGetSize(objects);
    sGameObjectID* objs = sNewArray(sGameObjectID, obj_count);
    memcpy(objs, objects, obj_count*sizeof(sGameObjectID));
	for (size_t i=0; i<obj_count; i++)
	{
		if (!objs[i]->fake_user && !objs[i]->scene)
        {
            printf("Удаляется sGameObject(%s)\n", objs[i]->name);
			sGameObjectDelete(objs[i]);
		} else {
            printf("sGameObject(%s) присоединён к сцене и %sимеет\n", objs[i]->name, objs[i]->fake_user ? "" : "не" );
        }
    	puts("");
	}
    sDelete(objs);
}
