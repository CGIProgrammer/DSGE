/*
 ============================================================================
 Name        : siberian.c
 Author      : Ivan
 Version     :
 Copyright   : Your copyright notice
 Description : Hello World in C, Ansi-style
 ============================================================================
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "engine.h"
#include "game_structures.h"
#include "linalg.h"
#include "2D_renderer/2D_renderer.h"
#include "2D_renderer/forms.h"

uint32_t cnt = 0;

void rotate_bone(void* obj)
{
	sObject* skel = obj;

	sObjectRotateGlobal3f(skel, 0.0, 0.0, 0.01);
	cnt++;
	if (cnt>314)
	{
		skel->behaviour = 0;
	}
	//LAPrint(bone->transform);
}

int cframe = 0;
int endframe = 569;

char namebuff[256];

void fluid(void* fl)
{
	sObject* obj = fl;
	sScene* scene = obj->scene;

	sprintf(namebuff, "fluid/frame_%03d", cframe);
	sMaterial* old_mat = obj->mesh->material;
	sSceneRemoveMesh(scene, obj->mesh->name);
	sMesh* new_mesh = sSceneAddMesh(scene, namebuff);
	new_mesh->material = old_mat;
	obj->mesh = new_mesh;
	//new_mesh->material = sSceneGetMaterial(scene, "Материал.004");
	cframe++;
	if (cframe==endframe)
	{
		obj->behaviour = (fptr)0;
	}
}

void moveRigToBody(void* ptr)
{
	sObject* obj = ptr;
	sObject* body = obj->data;
	obj->transform = body->transform_global;
}

char names[][2][256] =
{
		{"bDEF-spine.006", "ohead"},
		{"bDEF-spine.003", "ospine3"},
		{"bDEF-spine.002"  , "ospine2"},
		{"bDEF-spine"  , "ospine"},
		{"bDEF-upper_arm.L", "oupperArm.L"},
		{"bDEF-upper_arm.R", "oupperArm.R"},
		{"bDEF-forearm.L"  , "oforearm.L"},
		{"bDEF-forearm.R"  , "oforearm.R"},
		{"bDEF-thigh.L"    , "othigh.L"},
		{"bDEF-thigh.R"    , "othigh.R"},
		{"bDEF-shin.L"     , "oshin.L"},
		{"bDEF-shin.R"     , "oshin.R"}
};

sTexture cbm;

void set(sScene* scene)
{
	sSceneLoad(scene,"ragdoll_bicycle");
	//scene.camera.view_point = sSceneGetObject(&scene, "oКуб");
	sEngineSetActiveScene(scene);
	sSceneSetSkyTexture(scene, &cbm);
	sSkeleton* rig = sSceneGetObject(scene, "smetarig");
	sPlayerInit(scene, rig);
	sPlayerMouseLookOn(scene);

	rig = (sSkeleton*)sSceneAddObject(scene, "sBaseCharacter_rig");

	sRagdoll ragdoll;
	if (rig)
	{
		printf("%i\n", sRagdollInit(scene, &ragdoll, 1, ""));

		sSkeletonSetPlayAction(rig, "BaseCharacter_Action", 0, ACTION_PLAY, 45.0, 45.1, 0.01);
		sActionProcess(rig);
		sObjectPlaceChildren((sObject*)rig);

		for (int i=0; i<sizeof(names)/sizeof(names[0]); i++)
		{
			sBone* bone= sSkeletonGetBone(rig, names[i][0]);
			sObject* obj  = sSceneGetObject(scene, names[i][1]);
			sObjectRemoveParent(bone);
			sObjectSetParent(bone, obj, 1);
			sBoneSetAnimatedFlag(bone, 0);
			obj->mesh = 0;
		}
		for (int i=0; i<rig->bone_count; i++)
		{
			rig->bones[i].animated = 0;
		}
		sPhysicsJointSetAngle1Rate(ragdoll.joints[8], 1.0, 10.0);
	}
}

void reset(sScene* scene)
{
	if (sKeyboardGetKeyState(GLFW_KEY_ENTER)==1)
	{
		sSceneFree(scene);
		set(scene);
		scene->behaviour = (void(*)(void*))reset;
	}
}
void test();
int main(void)
{
	sShaderSetVersion("330");
	
	sRenderSetSSGI(1);
	sRenderSetBloom(1);
	sRenderSetMotionBlur(1);
	sRenderSetReflections(1);
	sRenderSetHDR(1);
	sEngineCreateWindow(0,0,1);
	sEngineSetSwapInterval(1);
	sEngineStartOpenGL();
	sScene scene;
	memset(&scene, 0, sizeof(scene));
	sTextureLoadCubemap(&cbm, "data/textures/cubemap/small_room.dds");

	sSceneLoad(&scene,"demo_ranch");
	//scene.camera.view_point = sSceneGetObject(&scene, "oКуб");
	sEngineSetActiveScene(&scene);
	sSceneSetSkyTexture(&scene, &cbm);
	sSkeleton* rig = sSceneGetObject(&scene, "sBaseCharacter_rig");
	if (rig)
		sSkeletonSetPlayAction(rig, "BaseCharacter_bicycle", 0, ACTION_LOOP, 30, 66, 1);
	sPlayerInit(&scene, 0);
	sPlayerMouseLookOn(&scene);

	/*fForm* list;
	list = fListCreate(300,100, 100, 300,0);
	//fListConstructor(&list, 300,100, 100, 300,0);
	fListAddItem(list, "kajsdhjk");
	fListAddItem(list, "kajsdhjk");
	fListAddItem(list, "kajsdhjk");
	fListAddItem(list, "kajsdhjk");*/
	sEngineStartLoop();
	return 0;
}
