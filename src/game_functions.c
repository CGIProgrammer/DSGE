/*
 * game_functions.c
 *
 *  Created on: 9 янв. 2018 г.
 *      Author: ivan
 */
#include "engine.h"
#include "game_structures.h"

typedef struct
{
	char ID;
	char ColorMap;
	char DataType;
	char ColorMapInfo[5];
	uint16_t x_origin;
	uint16_t y_origin;
	uint16_t width;
	uint16_t height;
	char BitsPerPixel;
	char description;
} TARGA;

uint32_t frame=0;
_Bool capture = 0;

typedef struct
{
	char object_name[64];
	char inventory_name[64];
	float condition;
	float weight;
	uint32_t count;
} sItem;

typedef struct
{

} sInventory;

void light_screenshot(sCamera* camera)
{
	glBindFramebuffer(GL_FRAMEBUFFER,camera->render_fb);
	TARGA image;
	memset(&image,0,sizeof(image));
	image.x_origin = 0;
	image.y_origin = 0;
	image.width = camera->width;
	image.height = camera->height;
	image.DataType = 2;
	image.BitsPerPixel = 24;
	sprintf(buff,"data/screenshots/screen %dx%d.tga",camera->width,camera->height);
	FILE* fp = fopen(buff,"wb");
	if (!fp)
	{
		printf("Screenshot failed\n");
		return;
	}
	char* data = malloc(camera->width*camera->height*3);
	glReadPixels(0,0,image.width,image.height,GL_RGB,GL_UNSIGNED_BYTE,data);
	fwrite(&image,sizeof(image),1,fp);
	fwrite(data,image.width*image.height,3,fp);
	fclose(fp);
	free(data);
}

sObject* dots[8];

void lookat(void* obj)
{
	sSkeleton* skeleton = obj;
	sBone* object = &skeleton->bones[4];
	sObject* targ = sSceneGetObject(object->scene,"otarget");
	sObject* elb = sSceneGetObject(object->scene,"oelbow");

	sObjectIK(object,elb,targ);
	float speed =0.02;
	float lft = ((sKeyboardGetKeyState(GLFW_KEY_LEFT)==GLFW_PRESS))*speed-((sKeyboardGetKeyState(GLFW_KEY_RIGHT)==GLFW_PRESS))*speed;
	targ->transform.a[11] += lft;
}

void sVehicleInit(sScene* scene,sVehicle4Wheel* vehicle,char* prefix)
{
	sObject** oSlider = (sObject**)&vehicle->fls;
	sObject** oWheel = (sObject**)&vehicle->flw;

	vehicle->breaks = 0;
	vehicle->gas = 0;
	vehicle->rpm = 0;
	vehicle->power = 14.0;

	vehicle->transmission = 0;

	if (vehicle->acceleration<=1.0)
	{
		vehicle->acceleration = 1.0;
	}
	if (vehicle->acceleration>10.0)
	{
		vehicle->acceleration = 10.0;
	}

	if (!vehicle->body)
	{
		sprintf(buff,"o%sBody",prefix);
		vehicle->body = sSceneGetObject(scene,buff);
		if (!vehicle->body) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
		dBodySetAutoDisableFlag(vehicle->body->body,0);
	}
	if (!vehicle->flw)
	{
		sprintf(buff,"o%sFLW",prefix);
		vehicle->flw = sSceneGetObject(scene,buff);
		if (!vehicle->flw) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}
	if (!vehicle->frw)
	{
		sprintf(buff,"o%sFRW",prefix);
		vehicle->frw = sSceneGetObject(scene,buff);
		if (!vehicle->frw) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}
	if (!vehicle->blw)
	{
		sprintf(buff,"o%sBLW",prefix);
		vehicle->blw = sSceneGetObject(scene,buff);
		if (!vehicle->blw) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}
	if (!vehicle->brw)
	{
		sprintf(buff,"o%sBRW",prefix);
		vehicle->brw = sSceneGetObject(scene,buff);
		if (!vehicle->brw) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}

	if (!vehicle->fls)
	{
		sprintf(buff,"o%sFLS",prefix);
		vehicle->fls = sSceneGetObject(scene,buff);
		if (!vehicle->fls) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}
	if (!vehicle->frs)
	{
		sprintf(buff,"o%sFRS",prefix);
		vehicle->frs = sSceneGetObject(scene,buff);
		if (!vehicle->frs) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}
	if (!vehicle->bls)
	{
		sprintf(buff,"o%sBLS",prefix);
		vehicle->bls = sSceneGetObject(scene,buff);
		if (!vehicle->bls) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}
	if (!vehicle->brs)
	{
		sprintf(buff,"o%sBRS",prefix);
		vehicle->brs = sSceneGetObject(scene,buff);
		if (!vehicle->brs) {fprintf(stderr,"%s not found\n",buff);exit(-1);};
	}

	vehicle->brw->mesh->material->friction = 1000.0;
	//vehicle->body->mesh->material->friction = 1.0;
	dReal fSk = 5000.0; // spring
	dReal fDk = 1000.0; // damper

	dReal fERP = 0.0166666 * fSk / ( 0.0166666 * fSk + fDk );
	dReal fCFM = 1.0 / ( 0.0166666 * fSk + fDk );

	vehicle->body->collisionGroups    =  0b10000;
	vehicle->body->collideWithGroups &= ~0b01000;

	for (int i=0;i<4;i++)
	{
		dMass mass;
		sMesh *mesh = oWheel[i]->mesh;
		dMassSetBoxTotal(&mass,oWheel[i]->physicsMass,mesh->bounding_box.a[0],mesh->bounding_box.a[1],mesh->bounding_box.a[2]);
		dBodySetMass(oWheel[i]->body,&mass);

		oWheel[i]->collisionGroups    =  0b01000;
		oWheel[i]->collideWithGroups &= ~0b10000;

		//dBodySetFiniteRotationMode(oWheel[i]->body,1);
		//dBodySetFiniteRotationAxis(oWheel[i]->body,0,0,1);

		dJointID sjoint = vehicle->joints[i+4] = dJointCreateSlider(scene->world,0);
		oSlider[i]->bodyIgnoring = vehicle->body->body;

		dJointAttach(sjoint,vehicle->body->body,oSlider[i]->body);
		dJointSetSliderAxis(sjoint,0.0,0.0,1.0);

		// Высота подвески
		dJointSetSliderParam(sjoint, dParamLoStop, 0.0);
		dJointSetSliderParam(sjoint, dParamHiStop, 0.0);

		dJointSetSliderParam(sjoint, dParamBounce, 1.0);

		dJointSetSliderParam(sjoint, dParamStopERP, fERP);
		dJointSetSliderParam(sjoint, dParamStopCFM, fCFM);
		dJointSetSliderParam(sjoint, dParamCFM, 0.999);
		dJointSetSliderParam(sjoint, dParamERP, 0.0001);

		//if (i&2)
		{
			dJointID wjoint = vehicle->joints[i] = dJointCreateHinge2(scene->world,0);
			dJointAttach(wjoint, oSlider[i]->body, oWheel[i]->body);
			oWheel[i]->bodyIgnoring = oSlider[i]->body;

			dJointSetHinge2Axis1(wjoint,0.0,0.0,-1.0);
			dJointSetHinge2Axis2(wjoint,-1+2*(i==1 || i==3),0,0);

			dJointSetHinge2Param(wjoint,dParamHiStop1, 0.0);
			dJointSetHinge2Param(wjoint,dParamLoStop1, 0.0);
			dJointSetHinge2Param(wjoint,dParamHiStop2, dInfinity);
			dJointSetHinge2Param(wjoint,dParamLoStop2,-dInfinity);
			dJointSetHinge2Param(wjoint, dParamStopERP, 0.999);
			dJointSetHinge2Param(wjoint, dParamStopCFM, 0.0001);
		}
	}
}

void sVehicleEngine(sVehicle4Wheel *vehicle)
{
	//float circle;// = (&vehicle->flw)[i]->mesh->bounding_box.a[0]*3.1415926535;
	float rate = 0.0;// = dJointGetHinge2Angle2Rate(vehicle->joints[i]);
	dReal radius, length, velocity;
	dGeomCylinderGetParams(vehicle->flw->geom, &radius, &length);

	vehicle->acceleration = 2.0;
	vehicle->power = 3.0;

	int whls=0;
	for (int i=0;i<4;i++)
	{
		float dir = (i&1) ? -1.0 : 1.0;
		if ((vehicle->drive_wheels>>i)&1)
		{
			rate+=dJointGetHinge2Angle2Rate(vehicle->joints[i])*dir;
			whls++;
		}
	}
	rate /= whls;

	velocity = rate * radius * 3.6;
	printf("velocity %lf\n", velocity);

	float tr[] = {-0.01, 0.01, 0.015, 0.02, 0.03, 0.05};
	int transmissions_count = sizeof(tr)/sizeof(tr[0]);
	vehicle->rpm = fabs(rate/tr[(int)vehicle->transmission]);


	if (vehicle->rpm>3500.0)
	{
		vehicle->transmission += vehicle->transmission<(transmissions_count-1) && vehicle->transmission;
	}
	if (vehicle->rpm<1500 && vehicle->transmission>0)
	{
		vehicle->transmission -= vehicle->transmission>1;
	}
	//vehicle->transmission += !vehicle->transmission;
	float rpm_increment = (6000-vehicle->rpm) * vehicle->acceleration;
	rpm_increment = rpm_increment>3000 ? 3000 : rpm_increment;
	rpm_increment = rpm_increment<0 ? 0 : rpm_increment;

	if (vehicle->gas > 0.0f)
	{
		vehicle->rpm += rpm_increment * sGetFrameTime() * vehicle->gas;
	}
	else
	{
		//vehicle->rpm /= powf(1.5, sGetFrameTime() * 60.0);
	}

	float dir;
	for (int i=0;i<4;i++)
	{
		if ((vehicle->drive_wheels>>i)&1)
		{
			dir = (i&1) ? -1.0 : 1.0;
			if (vehicle->breaks)
			{
				dJointSetHinge2Param( vehicle->joints[i], dParamFMax2, 300.0);
				dJointSetHinge2Param( vehicle->joints[i], dParamVel2, 0.0);
			}
			else
			{
				dJointSetHinge2Param( vehicle->joints[i], dParamFMax2, fabs(vehicle->power/tr[(int)vehicle->transmission]));
				dJointSetHinge2Param( vehicle->joints[i], dParamVel2, vehicle->rpm * tr[(int)vehicle->transmission] * dir);
			}
		}
	}
}

void sVehicleTurn(sVehicle4Wheel* vehicle,float amount,uint8_t wheels)
{
	for (int i=0;i<4;i++)
	{
		if (!((wheels>>i)&1)) return;
		dJointSetHinge2Param(vehicle->joints[i],dParamHiStop,amount);
		dJointSetHinge2Param(vehicle->joints[i],dParamLoStop,amount);
	}
}

void sVehicleSetSuspension(sVehicle4Wheel* vehicle,float spring,float dump,uint8_t wheels)
{
	dReal fERP = 0.016666 * spring / ( 0.016666 * spring + dump );
	dReal fCFM = 1.0 / ( 0.016666 * spring + dump );
	for (int i=0;i<4;i++)
	{
		if (!((wheels>>i)&1)) return;
		dJointSetHinge2Param(vehicle->joints[i],dParamSuspensionCFM,fCFM);
		dJointSetHinge2Param(vehicle->joints[i],dParamSuspensionERP,fERP);
		/*dJointSetHinge2Param(vehicle->joints[i],dParamCFM,0.005);
		dJointSetHinge2Param(vehicle->joints[i],dParamERP,0.5);
		dJointSetHinge2Param(vehicle->joints[i],dParamCFM2,0.05);
		dJointSetHinge2Param(vehicle->joints[i],dParamERP2,0.5);*/
	}
}

void sVehicleSetMaxSpeedKPH(sVehicle4Wheel* vehicle,float speed)
{
	vehicle->max_speed = fabs(speed)/3.6;
}
void sVehicleSetMaxTorque(sVehicle4Wheel* vehicle,float torque)
{
	vehicle->max_torque = fabs(torque);
}

void sVehicleSetTireFriction(sVehicle4Wheel* vehicle,float friction)
{
	vehicle->blw->mesh->material->friction = friction;
	vehicle->brw->mesh->material->friction = friction;
	vehicle->flw->mesh->material->friction = friction;
	vehicle->frw->mesh->material->friction = friction;
}

static laType getJointCoords2(sObject* obj1,sObject* obj2)
{
	laType o1m = obj1->transform_global;
	laType o2m = obj2->transform_global;

	return Mulf(Add(
			Vector(o1m.a[3]  + o1m.a[2]  * obj1->mesh->bounding_box.a[2]*0.5,
				   o1m.a[7]  + o1m.a[6]  * obj1->mesh->bounding_box.a[2]*0.5,
				   o1m.a[11] + o1m.a[10] * obj1->mesh->bounding_box.a[2]*0.5),

			Vector(o2m.a[3]  - o2m.a[2]  * obj2->mesh->bounding_box.a[2]*0.5,
				   o2m.a[7]  - o2m.a[6]  * obj2->mesh->bounding_box.a[2]*0.5,
				   o2m.a[11] - o2m.a[10] * obj2->mesh->bounding_box.a[2]*0.5)
	),0.5);
}

/*static laType getJointCoordsRelative2(sObject* obj1,sObject* obj2)
{
	laType o1m = obj1->transform_global;
	laType o2m = obj2->transform_global;

	return Sub(Mulf(Add(
			Vector(o1m.a[3]  + o1m.a[2]  * obj1->mesh->bounding_box.a[2]*0.5,
				   o1m.a[7]  + o1m.a[6]  * obj1->mesh->bounding_box.a[2]*0.5,
				   o1m.a[11] + o1m.a[10] * obj1->mesh->bounding_box.a[2]*0.5),

			Vector(o2m.a[3]  - o2m.a[2]  * obj2->mesh->bounding_box.a[2]*0.5,
				   o2m.a[7]  - o2m.a[6]  * obj2->mesh->bounding_box.a[2]*0.5,
				   o2m.a[11] - o2m.a[10] * obj2->mesh->bounding_box.a[2]*0.5)
	),0.5), Vector(obj1->transform_global.a[7], obj1->transform_global.a[7], obj1->transform_global.a[11]));
}*/

static laType getJontCoords(sObject* obj)
{
	return Vector(obj->transform_global.a[3]  + obj->transform_global.a[2]  * obj->mesh->bounding_box.a[2]*0.5,
				  obj->transform_global.a[7]  + obj->transform_global.a[6]  * obj->mesh->bounding_box.a[2]*0.5,
				  obj->transform_global.a[11] + obj->transform_global.a[10] * obj->mesh->bounding_box.a[2]*0.5);
}

/*static laType getJontCoordsRelative(sObject* obj)
{
	return Vector(obj->transform_global.a[2] *obj->mesh->bounding_box.a[2]*0.5,
				obj->transform_global.a[6] *obj->mesh->bounding_box.a[2]*0.5,
				obj->transform_global.a[10]*obj->mesh->bounding_box.a[2]*0.5);
}*/

void sRagdollServos(void* ptr)
{
	sObject* obj = ptr;
	sRagdoll* ragdoll = obj->data;
	dReal a1, a2, a;

	for (int i=0; i<11;i++)
	{
		dJointID j = ragdoll->joints[i];
		switch(dJointGetType(j))
		{
		case dJointTypeUniversal :
			dJointGetUniversalAngles(j, &a1, &a2);
			dJointSetUniversalParam(j, dParamVel,   -a1);
			dJointSetUniversalParam(j, dParamVel2,  -a2);
			dJointSetUniversalParam(j, dParamFMax,  50.0);
			dJointSetUniversalParam(j, dParamFMax2, 50.0);
		break;

		case dJointTypeHinge :
			a = dJointGetHingeAngle(j);
			dJointSetHingeParam(j, dParamVel,   -a);
			dJointSetHingeParam(j, dParamFMax,  50.0);
		break;
		default : break;
		}
	}
}

int sRagdollInit(sScene* scene,sRagdoll* ragdoll, _Bool autodetect,char* prefix)
{
	// В структуру sRagdoll можно добавлять объекты физической модели Rogdoll
	// Если autodetect равно true, то добавятся объекты с соответствующими именами
	if (autodetect)
	{
		sprintf(buff,"o%shead",prefix);
		ragdoll->head = sSceneAddObject(scene,buff);	// Добавление объекта на сцену

		sprintf(buff,"o%sspine3",prefix);
		ragdoll->spine1 = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sspine2",prefix);
		ragdoll->spine2 = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sspine",prefix);
		ragdoll->spine3 = sSceneAddObject(scene,buff);

		sprintf(buff,"o%supperArm.L",prefix);
		ragdoll->lShoulder = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sforearm.L",prefix);
		ragdoll->lForearm = sSceneAddObject(scene,buff);

		sprintf(buff,"o%supperArm.R",prefix);
		ragdoll->rShoulder = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sforearm.R",prefix);
		ragdoll->rForearm = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sthigh.L",prefix);
		ragdoll->lLeg = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sshin.L",prefix);
		ragdoll->lKnee = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sthigh.R",prefix);
		ragdoll->rLeg = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sshin.R",prefix);
		ragdoll->rKnee = sSceneAddObject(scene,buff);

		sprintf(buff,"o%sfoot.L",prefix);
		ragdoll->lFoot = sSceneAddObject(scene,buff);
		if (!ragdoll->lFoot)
		{
			printf("%s not found\n", buff);
		}

		sprintf(buff,"o%sfoot.R",prefix);
		ragdoll->rFoot = sSceneAddObject(scene,buff);
		if (!ragdoll->rFoot)
		{
			printf("%s not found\n", buff);
		}
	}

	memset(ragdoll->joints, 0, sizeof(ragdoll->joints));

	/*
	 * sPhysicsCreateAnchor - создание шарнирного соединения
	 * Аргументы
	 * 1 - группа сочленений
	 * 2 - первое тело
	 * 3 - второе тело
	 * 4 - координаты точки соединения
	 * 5 - направление первой оси вращения
	 * 6 - направление второй оси вращения
	 * 7 - относительные/неотносительные координаты соединения
	 */

	/*
	 * sPhysicsCreateCardan - создание карданного соединения
	 * Аргументы
	 * 1 - группа сочленений
	 * 2 - первое тело
	 * 3 - второе тело
	 * 4 - минимальный угол  (ограничение)
	 * 5 - максимальный угол (ограничение)
	 * 6 - координаты точки соединения
	 * 7 - направление оси вращения
	 * 8 - относительные/неотносительные координаты соединения
	 */

	if (!ragdoll->spine1 || !ragdoll->spine2 || !ragdoll->spine3) return 1;
	ragdoll->joints[0] = sPhysicsCreateAnchor(ragdoll->spine1,ragdoll->spine2,-3.1415926535/4, 3.1415926535/4,getJointCoords2(ragdoll->spine1,ragdoll->spine2),Vector(1.0,0.0,0.0), 0);
	ragdoll->spine1->bodyIgnoring = ragdoll->spine2->body;
	ragdoll->joints[1] = sPhysicsCreateAnchor(ragdoll->spine2,ragdoll->spine3,-3.1415926535/4*0.2, 3.1415926535/4*0.2,getJointCoords2(ragdoll->spine2,ragdoll->spine3),Vector(1.0,0.0,0.0), 0);
	ragdoll->spine2->bodyIgnoring = ragdoll->spine3->body;

	if (ragdoll->lForearm && ragdoll->lShoulder)
	{
		// Создание левого локтевого сустава
		ragdoll->joints[2] = sPhysicsCreateAnchor(ragdoll->lForearm,ragdoll->lShoulder, 0.0, -3.1415926535*0.75,getJointCoords2(ragdoll->lForearm, ragdoll->lShoulder),Vector(-1.0,0.0,0.0), 0);
		ragdoll->lShoulder->bodyIgnoring = ragdoll->lForearm->body;
	}
	if (ragdoll->rForearm && ragdoll->rShoulder)
	{
		// Создание правого локтевого сустава
		ragdoll->joints[3] = sPhysicsCreateAnchor(ragdoll->rForearm,ragdoll->rShoulder, 0.0, -3.1415926535*0.75,getJointCoords2(ragdoll->rForearm, ragdoll->rShoulder),Vector(-1.0,0.0,0.0), 0);
		ragdoll->rShoulder->bodyIgnoring = ragdoll->rForearm->body;
	}

	if (ragdoll->lShoulder)	// Создание левого плечевого сустава
		ragdoll->joints[4] = sPhysicsCreateCardan(ragdoll->spine1,ragdoll->lShoulder,getJontCoords(ragdoll->lShoulder),Vector( 1.0,0.0,0.0),Vector(0.0, 1.0,0.0), 0);
	if (ragdoll->rShoulder)	// Создание правого плечевого сустава
		ragdoll->joints[5] = sPhysicsCreateCardan(ragdoll->spine1,ragdoll->rShoulder,getJontCoords(ragdoll->rShoulder),Vector( 1.0,0.0,0.0),Vector(0.0, 1.0,0.0), 0);
	if (ragdoll->head)		// Создание шеи (присоединение головы)
		ragdoll->joints[6] = sPhysicsCreateCardan(ragdoll->spine1,ragdoll->head,getJointCoords2(ragdoll->spine1,ragdoll->head),Vector(1.0,0.0,0.0),Vector(0.0,1.0,0.0), 0);

	if (ragdoll->lKnee && ragdoll->lLeg)
	{
		// Левый коленный сустав
		ragdoll->joints[7] = sPhysicsCreateAnchor(ragdoll->lKnee,ragdoll->lLeg, 0.0, 3.1415926535*0.75,getJointCoords2(ragdoll->lKnee, ragdoll->lLeg),Vector(-1.0,0.0,0.0), 0);
		ragdoll->lKnee->bodyIgnoring = ragdoll->lLeg->body;
	}
	if (ragdoll->rKnee && ragdoll->rLeg)
	{
		// Правый коленный сустав
		ragdoll->joints[8] = sPhysicsCreateAnchor(ragdoll->rKnee,ragdoll->rLeg, 0.0, 3.1415926535*0.75,getJointCoords2(ragdoll->rKnee, ragdoll->rLeg),Vector(-1.0,0.0,0.0), 0);
		ragdoll->rKnee->bodyIgnoring = ragdoll->rLeg->body;
	}

	if (ragdoll->lLeg)
	{
		// Левый бедренный сустав
		ragdoll->joints[9]  = sPhysicsCreateCardan(ragdoll->spine3,ragdoll->lLeg,getJontCoords(ragdoll->lLeg),Vector(1.0,0.0,0.0),Vector(0.0,1.0,0.0), 0);
		ragdoll->lLeg->bodyIgnoring = ragdoll->spine3->body;
	}
	if (ragdoll->rLeg)
	{
		// Правый бедренный сустав
		ragdoll->joints[10] = sPhysicsCreateCardan(ragdoll->spine3,ragdoll->rLeg,getJontCoords(ragdoll->rLeg),Vector(1.0,0.0,0.0),Vector(0.0,1.0,0.0), 0);
		ragdoll->rLeg->bodyIgnoring = ragdoll->spine3->body;
	}

	if (ragdoll->lFoot && ragdoll->lKnee)
	{
		// Левая стопа
		laType pos = Vector(ragdoll->lKnee->transform_global.a[3],ragdoll->lKnee->transform_global.a[7], ragdoll->lKnee->transform_global.a[11]);
		pos = Sub(pos,
				Mulf(Vector(ragdoll->lKnee->transform_global.a[2],ragdoll->lKnee->transform_global.a[6], ragdoll->lKnee->transform_global.a[10]), ragdoll->lKnee->mesh->bounding_box.a[2]*0.5)
		);
		ragdoll->joints[11] = sPhysicsCreateAnchor( ragdoll->lFoot, ragdoll->lKnee, -3.1415926535*0.5, 3.1415926535*0.5, pos, Vector(1.0, 0.0, 0.0), 0);
	}

	if (ragdoll->rFoot && ragdoll->rKnee)
	{
		// Правая стопа
		laType pos = Vector(ragdoll->rKnee->transform_global.a[3],ragdoll->rKnee->transform_global.a[7], ragdoll->rKnee->transform_global.a[11]);
		pos = Sub(pos,
				Mulf(Vector(ragdoll->rKnee->transform_global.a[2],ragdoll->rKnee->transform_global.a[6], ragdoll->rKnee->transform_global.a[10]), ragdoll->rKnee->mesh->bounding_box.a[2]*0.5)
		);
		ragdoll->joints[12] = sPhysicsCreateAnchor( ragdoll->rFoot, ragdoll->rKnee, -3.1415926535*0.5, 3.1415926535*0.5, pos, Vector(1.0, 0.0, 0.0), 0);
	}

	// Настройка ограничений суставов
	dJointSetUniversalParam(ragdoll->joints[9], dParamLoStop, radians(-135));
	dJointSetUniversalParam(ragdoll->joints[9], dParamHiStop, radians( 15));
	dJointSetUniversalParam(ragdoll->joints[9], dParamLoStop2,radians(-45));
	dJointSetUniversalParam(ragdoll->joints[9], dParamHiStop2,radians( 90));

	dJointSetUniversalParam(ragdoll->joints[10],dParamLoStop, radians(-135));
	dJointSetUniversalParam(ragdoll->joints[10],dParamHiStop, radians( 15));
	dJointSetUniversalParam(ragdoll->joints[10],dParamLoStop2,radians(-90));
	dJointSetUniversalParam(ragdoll->joints[10],dParamHiStop2,radians( 45));

	dJointSetUniversalParam(ragdoll->joints[4],dParamLoStop, radians(-90));
	dJointSetUniversalParam(ragdoll->joints[4],dParamHiStop, radians( 90));

	dJointSetUniversalParam(ragdoll->joints[5],dParamLoStop, radians(-90));
	dJointSetUniversalParam(ragdoll->joints[5],dParamHiStop, radians( 90));

	/*for (int i=0; i<sizeof(ragdoll->joints)/sizeof(ragdoll->joints[0]); i++)
	{
		for (int j=0; j<sizeof(ragdoll->joints)/sizeof(ragdoll->joints[0])-1; j++)
		{
			if ((!(ragdoll->joints[j] && ragdoll->joints[j+1]) && ragdoll->joints[j] < ragdoll->joints[j+1]) ||
					((ragdoll->joints[j] && ragdoll->joints[j+1]) && (dJointGetType(ragdoll->joints[j]) < dJointGetType(ragdoll->joints[j+1]))))
			{
				dJointID joint = ragdoll->joints[j];
				ragdoll->joints[j] = ragdoll->joints[j+1];
				ragdoll->joints[j+1] = joint;
			}
		}
	}*/
	return 0;
}

void sBicycleAssemble(sObject* steer, sObject* fwheel, sObject* bwheel, sObject* body)
{
	fwheel->bodyIgnoring = steer->body;
	bwheel->bodyIgnoring = body->body;
	steer->bodyIgnoring = body->body;

	sPhysicsCreateAnchor( fwheel, steer, -10, 10, Vector(0,0,0), Vector(fwheel->transform_global.a[2],fwheel->transform_global.a[6],fwheel->transform_global.a[10]), 1);
	sPhysicsCreateAnchor( bwheel, body,  -10, 10, Vector(0,0,0), Vector(bwheel->transform_global.a[2],bwheel->transform_global.a[6],bwheel->transform_global.a[10]), 1);
	sPhysicsCreateAnchor( steer,  body,  -10, 10, Vector(0,0,0), Vector(0.0, 0.0, 1.0), 1);
}

static float angle_x = 3.1415926535,angle_y = 3.1415926535/2;
static float angle_dx = 0.0,angle_dy = 0.0;
static float look_impct_x = 0.0;
static float look_impct_y = 0.0;
static float look_impct_z = 0.0;
static float look_impact_inertia = 0.9;
static float look_inertia = 0.85;
static float walk_inertia = 0.9;
static float walk_max_angle = 0.2;
static laType walk_vector;
laType walk_speed_vector;
static uint8_t walk_step_trigger;
uint8_t walk_step;
static laType initial_hud_position;
static laType initial_cam_position;
static double rocking_timer = 0.0;
static sObject *character = 0;
static sObject *sun = 0;
static sObject *sky;

uint8_t upstairs = 0;
uint8_t jump = 0;
static uint8_t mouse_look = 0;

void sPlayerSetImpact(float x,float y,float z)
{
	look_impct_x = x;
	look_impct_y = y;
	look_impct_z = z;
}

static void player_look(sObject *object)
{
	sCamera *camera = (sCamera*)object;
	sObject *player = camera->parent;
	sSkeleton *HUD = 0;
	for (int i=0;i<object->child_count;i++)
	{
		if (((sObject*)object->children[i])->name[0] == 's')
		{
			HUD = (sSkeleton*)object->children[i];
		}
	}
	float ft_coeff = sGetFrameTime()*60.0;
	if (((sObject*)camera->parent)->name[0]=='b')
		camera = camera->parent;
	if (camera->name[0]=='b')
	{
		sBone *bone = (sBone*)camera;
		sObject *skel = bone->skeleton;
		player = skel->parent;
	}
	float cur_p[2],cur_x,cur_y;
	if (mouse_look)
	{
		sGetMousePosition(cur_p);
		cur_x = cur_p[0]-320;
		cur_y = cur_p[1]-240;
		sMouseSetPosition(320.0,240.0);
	}
	else
	{
		cur_x = 0.0;
		cur_y = 0.0;
	}
	angle_dx -= cur_x;
	angle_dy += cur_y;
	angle_x += angle_dx*0.0002 * ft_coeff;
	angle_y -= angle_dy*0.0002 * ft_coeff;

	angle_dx *= pow(look_inertia,ft_coeff);
	angle_dy *= pow(look_inertia,ft_coeff);
	look_impct_x *= pow(look_impact_inertia,ft_coeff);
	look_impct_y *= pow(look_impact_inertia,ft_coeff);
	look_impct_z *= pow(look_impact_inertia,ft_coeff);

	if (angle_y>3.1415926535)
	{
		angle_y = 3.1415926535;
	}
	if (angle_y<0.0)
	{
		angle_y = 0.0;
	}

	float rocks = sin(rocking_timer);
	float rockc = cos(rocking_timer*1.75);

	camera->transform = initial_cam_position;
	sObjectMoveLocal3fv(camera,Vector(rocks*0.035,(-1.0+rockc)*0.035,0.0));
	sObjectSetRotation3f(camera,angle_y+look_impct_y,0.0,0.0);
	sObjectRotateLocal3f(camera,0,0,look_impct_z+cos(rocking_timer*1.75/2.0)*0.005);
	sObjectSetRotation3f(player,0.0,0.0,angle_x+look_impct_x);

	laType directionX,directionY,direction = player->transform_global;
	directionX = Vector(direction.a[0],direction.a[4],0.0);
	directionY = Vector(direction.a[1],direction.a[5],0.0);

	if (sPhysicsCSGetHitObjectCount(player))
	{
		jump = 0;
	}
	if (sPhysicsCSGetHitObjectCount(player))
	{
		sPhysicsContact cont = player->ray.contacts[0];
		for (uint32_t i=1;i<player->ray.contactsCount;i++)
		{
			if (player->ray.contacts[i].position[2] < cont.position[2])
			{
				cont = player->ray.contacts[i];
			}
		}

		jump = 0;
		float norm = 0.0;
		if (player->ray.contactsCount)
		{
			norm = cont.normal[2];
		}

		/*laType hit_position = Vector(player->collider.contacts[0].position[0] - player->transform.a[3],
									 player->collider.contacts[0].position[1] - player->transform.a[7],
									 player->collider.contacts[0].position[2] - player->transform.a[11]);*/
		laType hitNormal   = Vector(cont.normal[0],cont.normal[1],cont.normal[2]);
		laType forwardVector = Vector(player->transform_global.a[1],player->transform_global.a[5],player->transform_global.a[9]);

		//LAPrint(hit_position);

		//hit_position.a[2] = 0.0;
		if (hitNormal.a[2] < 0.0)
		{
			hitNormal.a[0] = -hitNormal.a[0];
			hitNormal.a[1] = -hitNormal.a[1];
			hitNormal.a[2] = -hitNormal.a[2];
		}
		//LAPrint(hitNormal);
		directionX = Crossn(forwardVector,hitNormal);
		directionY = Crossn(hitNormal,directionX);

		walk_vector = Mulf(directionX, (sKeyboardGetKeyState(GLFW_KEY_D)-sKeyboardGetKeyState(GLFW_KEY_A)));
		walk_vector = Add(walk_vector, Mulf(directionY, (sKeyboardGetKeyState(GLFW_KEY_W)-sKeyboardGetKeyState(GLFW_KEY_S))));
		Normalize(&walk_vector);
		walk_vector.a[2] -= 0.09;
		walk_vector.a[2] *= 1.5;

		if (norm>walk_max_angle || walk_vector.a[2]<0.0)
		{
			walk_speed_vector = Add(walk_speed_vector,Mulf(walk_vector,0.4));
			walk_speed_vector = Mulf(walk_speed_vector,pow(walk_inertia,ft_coeff));
		}
		//hit_position.a[2] = 0.0;
		upstairs = 0;
		for (uint32_t i=0; i<sPhysicsCSGetHitObjectCount(player); i++)
		{
			sObject* obj = player->collider.contacts[i].object;
			if (obj->mesh && obj->mesh->material && !strcmp(obj->mesh->material->name, "stairs_trigger"))
			{
				upstairs = 1;
				puts("Upstairs");
				break;
			}
		}
		if (!upstairs)
		{
			sPhysicsSetSpeedGlobal(player,Vector(walk_speed_vector.a[0]*ft_coeff,walk_speed_vector.a[1]*ft_coeff,(walk_speed_vector.a[2])*ft_coeff),0b011);
		}
		else
		{
			sObject* cam = (sObject*)&((sScene*)player->scene)->camera;
			laType view_mat = sCameraGetTransform((sCamera*)cam);
			float ups = Dotn(Vector(view_mat.a[1], view_mat.a[5], view_mat.a[9]), directionY);
			ups *= -3.0;
			sPhysicsSetSpeedGlobal(player,Vector(walk_speed_vector.a[0]*ft_coeff,walk_speed_vector.a[1]*ft_coeff, ups), 0b111);
		}

		for (uint32_t i=0;i<player->collider.contactsCount;i++)
		{
			laType hit_position = Vector(player->collider.contacts[0].position[0] - player->transform.a[3],
										 player->collider.contacts[0].position[1] - player->transform.a[7],
										 player->collider.contacts[0].position[2] - player->transform.a[11]);

			Normalize(&hit_position);
			if (Dot(hit_position,Vector(0.0,0.0,1.0))<0.2 && sKeyboardGetKeyState(GLFW_KEY_SPACE))
			{
				jump = 1;
				break;
			}
		}
		if (jump)
		{
			sPhysicsSetSpeedZLocal(player,5.0);
		}
	}

	{
		if (HUD)
			HUD->transform = initial_hud_position;
		float walk_speed = sqrtf(sqrtf(powf(walk_speed_vector.a[0],2.0)+powf(walk_speed_vector.a[1],2.0))*0.175*ft_coeff);
		laType rocking = Vector(rocks*0.01,0.0,(-1.0+rockc)*0.01);

		walk_step = rockc<-0.75 && !walk_step_trigger;
		walk_step_trigger = rockc<-0.75;

		rocking = Mulf(rocking,walk_speed);
		if (HUD)
			sObjectMoveLocal3fv(HUD,rocking);
		rocking_timer += ft_coeff*walk_speed * (sPhysicsCSGetHitObjectCount(player) ? 0.2 : 0.05);
		walk_speed_vector.a[2] *= pow(walk_inertia,ft_coeff);
	}

	//sObject *sky = sSceneGetObject(player->scene,"osky");
	if (sky)
	{
		sObjectMoveGlobal3fv(sky,sObjectGetVectorTo(sky, camera));
	}
	if (sun)
	{
		sun->transform.a[3]  = sun->transform.a[2]  * ((sLight*)sun)->zFar * 0.333;
		sun->transform.a[7]  = sun->transform.a[6]  * ((sLight*)sun)->zFar * 0.333;
		sun->transform.a[11] = sun->transform.a[10] * ((sLight*)sun)->zFar * 0.333;
		sObjectMoveGlobal3fv(sun, Vector(player->transform.a[3], player->transform.a[7], player->transform.a[11]));
	}
}

void sPlayerMouseLookOff(sScene* scene)
{
	mouse_look = 0;
	scene->camera.behaviour = 0;
}

void sPlayerMouseLookOn(sScene* scene)
{
	mouse_look = 1;
	sMouseSetPosition(320.0,240.0);
	scene->camera.behaviour = (fptr)player_look;
}

void sVehicleController(sVehicle4Wheel *sVehicleActive, void* cam)
{
	sObject *body = sVehicleActive->body;
	sCamera* camera = cam;

	if (camera)
	{
		laType vect = GetVectorTo(body->transform, camera->transform);

		float len = fiSqrt(vect.a[0]*vect.a[0] + vect.a[1]*vect.a[1]);
		if (len<1)
		{
			vect.a[0] = vect.a[0] * len * 4.0;
			vect.a[1] = vect.a[1] * len * 4.0;
			vect.a[2] = 1.0;
		}

		camera->transform.a[3 ] = body->transform.a[ 3] + vect.a[0];
		camera->transform.a[7 ] = body->transform.a[ 7] + vect.a[1];
		camera->transform.a[11] = body->transform.a[11] + vect.a[2];

		camera->transform = LookAt(camera->transform, Add(body->transform_global, Vector(0.0, 0.0, 0.8)), laZ, laZ);

		if (sun)
		{
			sun->transform.a[3]  = sun->transform.a[2]  * ((sLight*)sun)->zFar * 0.333;
			sun->transform.a[7]  = sun->transform.a[6]  * ((sLight*)sun)->zFar * 0.333;
			sun->transform.a[11] = sun->transform.a[10] * ((sLight*)sun)->zFar * 0.333;
			sObjectMoveGlobal3fv(sun, Vector(body->transform.a[3], body->transform.a[7], body->transform.a[11]));
		}
	}
	laType vel = sPhysicsGetLinearVelocity(sVehicleActive->body);
	sVehicleActive->drive_wheels = 0b1100;

	if (sVehicleActive->transmission==1 && sKeyboardGetKeyState(GLFW_KEY_DOWN) && fabs(sVehicleActive->rpm)<20.0)
	{
		sVehicleActive->transmission = 0;
		//sVehicleActive->rpm = 550;
	}
	if (sVehicleActive->transmission==0 && sKeyboardGetKeyState(GLFW_KEY_UP) && fabs(sVehicleActive->rpm)<20.0)
	{
		sVehicleActive->transmission = 1;
		//sVehicleActive->rpm = 550;
	}

	sVehicleEngine(sVehicleActive);

	printf("transmission %hhi; RPM %f; vel %f\n",sVehicleActive->transmission,sVehicleActive->rpm, Length(vel)*3.6);

	if (sVehicleActive->transmission>0)
	{
		sVehicleActive->gas = sKeyboardGetKeyState(GLFW_KEY_UP) ? 1.0 : 0.0;
		sVehicleActive->breaks = sKeyboardGetKeyState(GLFW_KEY_DOWN);
	}
	else
	{
		sVehicleActive->gas = sKeyboardGetKeyState(GLFW_KEY_DOWN) ? 1.0 : 0.0;
		sVehicleActive->breaks = sKeyboardGetKeyState(GLFW_KEY_UP);
	}

	if (sKeyboardGetKeyState(GLFW_KEY_LEFT))
	{
		sVehicleTurn(sVehicleActive,  0.5, 0b0011);
	}
	if (sKeyboardGetKeyState(GLFW_KEY_RIGHT))
	{
		sVehicleTurn(sVehicleActive, -0.5, 0b0011);
	}
	if (!sKeyboardGetKeyState(GLFW_KEY_RIGHT) && !sKeyboardGetKeyState(GLFW_KEY_LEFT))
	{
		sVehicleTurn(sVehicleActive, 0.0, 0b0011);
	}
}

/*void sRacerInit(sScene *scene,sVehicle4Wheel *vehicle)
{
	car_body = vehicle->body;
	car_body->behaviour = (fptr)car_controller;
}*/

void sPlayerInit(sScene *scene,sSkeleton *skeleton)
{
	sMouseSetPosition(320.0,0.0);
	sObject *capsule = sCalloc(sizeof(sObject),1);
	sObject *ray = sCalloc(sizeof(sObject),1);
	sCamera *camera = &scene->camera;
	camera->behaviour = (fptr)player_look;
	strcpy(capsule->name,"oPlayer");
	strcpy(ray->name,"oPlayerRay");
	capsule->hash = S_Name2hash(capsule->name);
	ray->hash 	  = S_Name2hash(ray->name);

	scene->gobjects = sRealloc(scene->gobjects,sizeof(GameObject*)*(scene->gobjects_count+2));
	scene->objects = sRealloc(scene->objects,sizeof(sObject*)*(scene->objects_count+2));
	scene->gobjects[scene->gobjects_count++] = (GameObject*)capsule;
	scene->objects[scene->objects_count++] = capsule;

	scene->gobjects[scene->gobjects_count++] = (GameObject*)ray;
	scene->objects[scene->objects_count++] = ray;

	capsule->transform = capsule->transform_global = capsule->transform_global_previous = Add(Identity,Vector(0.0,0.0,0.9));
	camera->transform = camera->transform_global = camera->transform_global_previous = Add(Identity,Vector(-0.05,-0.01,1.75));
	sObjectRotateLocal3f(capsule, 0.0,0.0,3.1415926535);
	sObjectRotateLocal3f(camera, -3.1415926535/2.0,0.0,3.1415926535);
	if (skeleton)
	{
		//sBone *head = sSkeletonGetBone(skeleton,"bchest");
		sObjectSetParent(skeleton,camera,1);
		sObjectSetParent(camera,capsule,1);
		//head->transformable = 0;
	}
	else
	{
		sObjectSetParent(camera,capsule,1);
	}

	ray->scene = scene;
	ray->physicsType = 0;
	ray->mesh = 0;
	ray->hidden = 0;
	ray->transform_global = ray->transform = Identity;
	ray->parent = 0;
	ray->children = 0;
	ray->bodyIgnoring = (void*)-1;
	ray->collisionGroups = 1;
	ray->collideWithGroups = ~0;
	sObjectSetParent(ray,camera,0);
	sPhysicsRSInit(ray,3.0);
	ray->ray.dir = rayX;

	initial_cam_position = camera->transform;
	sMesh cap_mesh;
	cap_mesh.bounding_box = Vector(0.9,0.9,1.75); // 0.6, 0.6, 1.75
	capsule->scene= scene;
	capsule->mesh = &cap_mesh;
	capsule->physicsMass = 2.0;
	capsule->collisionGroups = 1;
	capsule->collideWithGroups = ~0;
	capsule->physicsShape = 0;
	capsule->physicsType = 3;
	capsule->bodyIgnoring = (void*)-1;
	sPhysicsAttach(capsule);
	sPhysicsCSInit(capsule);
	sPhysicsRSInit(capsule,1.0);
	capsule->ray.dir = rayZn;
	walk_vector = Vector(0.0,0.0,0.0);
	walk_speed_vector = Vector(0.0,0.0,0.0);
	sPhysicsAutoDisable(capsule,0);
	capsule->mesh = 0;
	capsule->hidden = 0;
	if (skeleton)
		initial_hud_position = skeleton->transform;
	character = capsule;
}

sObject* sMobInit(sScene *scene,sSkeleton *skin, char *name, laType bbox)
{
	sObject *capsule = sCalloc(sizeof(sObject),1);
	strcpy(capsule->name,name);
	capsule->hash = S_Name2hash(capsule->name);

	scene->gobjects = sRealloc(scene->gobjects,sizeof(GameObject*)*(scene->gobjects_count+1));
	scene->objects = sRealloc(scene->objects,sizeof(sObject*)*(scene->objects_count+1));
	scene->gobjects[scene->gobjects_count] = (GameObject*)capsule;
	scene->objects[scene->objects_count] = capsule;
	scene->gobjects_count++;
	scene->objects_count++;
	capsule->transform = capsule->transform_global = capsule->transform_global_previous = skin->transform_global;//Add(Identity,Vector(0.0,0.0,0.9));
	skin->transform = skin->transform_global = Identity;
	sObjectSetParent(skin,capsule,0);

	sMesh cap_mesh;
	cap_mesh.bounding_box = bbox;
	capsule->scene= scene;
	capsule->mesh = &cap_mesh;
	capsule->physicsMass = 2.0;
	capsule->collisionGroups = 1;
	capsule->collideWithGroups = ~0;
	capsule->physicsShape = 0;
	capsule->physicsType = 3;
	capsule->bodyIgnoring = (void*)-1;
	sPhysicsAttach(capsule);
	sPhysicsCSInit(capsule);
	sPhysicsRSInit(capsule,0.9);
	capsule->ray.dir = rayZn;
	walk_vector = Vector(0.0,0.0,0.0);
	walk_speed_vector = Vector(0.0,0.0,0.0);
	sPhysicsAutoDisable(capsule,0);
	capsule->mesh = 0;
	capsule->hidden = 0;
	/*if (skeleton)
		initial_hud_position = skeleton->transform;*/
	return capsule;
}

sObject* sCharacterInit(sScene *scene,sSkeleton *skin, char *name)
{
	sObject *capsule = sCalloc(sizeof(sObject),1);
	strcpy(capsule->name,name);
	capsule->hash = S_Name2hash(capsule->name);

	scene->gobjects = sRealloc(scene->gobjects,sizeof(GameObject*)*(scene->gobjects_count+1));
	scene->objects = sRealloc(scene->objects,sizeof(sObject*)*(scene->objects_count+1));
	scene->gobjects[scene->gobjects_count] = (GameObject*)capsule;
	scene->objects[scene->objects_count] = capsule;
	scene->gobjects_count++;
	scene->objects_count++;
	capsule->transform = capsule->transform_global = capsule->transform_global_previous = Add(Identity,Vector(0.0,0.0,0.9));

	sObjectSetParent(skin,capsule,1);

	sMesh cap_mesh;
	cap_mesh.bounding_box = Vector(0.6,0.6,1.75);
	capsule->scene= scene;
	capsule->mesh = &cap_mesh;
	capsule->physicsMass = 2.0;
	capsule->collisionGroups = 1;
	capsule->collideWithGroups = ~0;
	capsule->physicsShape = 0;
	capsule->physicsType = 3;
	capsule->bodyIgnoring = (void*)-1;
	sPhysicsAttach(capsule);
	sPhysicsCSInit(capsule);
	sPhysicsRSInit(capsule,0.9);
	capsule->ray.dir = rayZn;
	walk_vector = Vector(0.0,0.0,0.0);
	walk_speed_vector = Vector(0.0,0.0,0.0);
	sPhysicsAutoDisable(capsule,0);
	capsule->mesh = 0;
	capsule->hidden = 0;
	/*if (skeleton)
		initial_hud_position = skeleton->transform;*/
	return capsule;
}

sRagdoll rg;
sVehicle4Wheel volga = {0,0,0,0,0,0,0,0,0};

sTexture sCubeMap;


char ssbttn=0;
char str=0;
static int ml = 1;
void sSceneFunctionLoop(sScene* scene)
{
	sun = sSceneGetObject(scene,"lsun");
	if (sKeyboardGetKeyState(GLFW_KEY_F12)==1)
	{
		time_t rawtime;
	    struct tm * timeinfo;

		time (&rawtime);
		timeinfo = localtime (&rawtime);
		
		puts(asctime(timeinfo));
		sCameraTakeScreenshot(&scene->camera, asctime(timeinfo));
	}
	if (sKeyboardGetKeyState(GLFW_KEY_F1)==1)
	{
		ml = !ml;
		if (ml)
		{
			sPlayerMouseLookOn(scene);
		}
		else
		{
			sPlayerMouseLookOff(scene);
		}
		
	}
	//sObjectRotateGlobal3f(&scene->camera, 0.0,0.0,-0.01);
	//LAPrint(scene->camera.transform_global);
	/*if (sKeyboardGetKeyState(GLFW_KEY_F12)>ssbttn)
	{
		if (str)
		{
			puts("Close stream");
			close_videostream();
		}
		else
		{
			puts("Open stream");
			open_videostream();
		}
		str = !str;
	}
	if (str)
	{
		stream_write_frame();
	}
	ssbttn = sKeyboardGetKeyState(GLFW_KEY_F12);*/

	/*if (character)
	{
		player_look((sObject*)&scene->camera);
	}*/
	//player_look(scene);
	//sObjectRotateLocal3f(sSceneGetObject(scene,"oPlayer"),0.0,0.0,0.01);
	//mouselook(scene);
	//sSkeleton* skeleton = sSceneGetObject(scene,"sSkellet");
	//if (skeleton)
	//	sActionProcess(skeleton);

	//sObject* obj = sSceneGetObject(scene,"oPlayer");

	/*if (obj->ray.contactsCount)
		printf("%.3f %.3f %.3f\n",obj->ray.contacts[0].position[0],
								  obj->ray.contacts[0].position[1],
								  obj->ray.contacts[0].position[2]);*/

	//sVehicleThrottle(&volga,100.0,0b1100);
	//dReal rate1 = dJointGetHinge2Angle2Rate(volga.joints[2]);
	//dReal rate2 = dJointGetHinge2Angle2Rate(volga.joints[3]);

	//dJointAddHinge2Torques(volga.joints[2],0.0, 10000.0/(80.0+rate1)*(sGetFrameTime()/0.016666));
	//dJointAddHinge2Torques(volga.joints[3],0.0,-10000.0/(80.0+rate2)*(sGetFrameTime()/0.016666));
}
#include "2D_renderer/2D_renderer.h"
void sPrintSizeOfAllGameStrictures(void)
{
	printf("sScene %lu\n",sizeof(sScene));
	printf("sObject %lu\n",sizeof(sObject));
	printf("sPhysicsContact %lu\n",sizeof(sPhysicsContact));
	printf("sPhysicsCS %lu\n",sizeof(sPhysicsCS));
	printf("sPhysicsRS %lu\n",sizeof(sPhysicsRS));
	printf("sMesh %lu\n",sizeof(sMesh));
	printf("sLight %lu\n",sizeof(sLight));
	printf("sCamera %lu\n",sizeof(sCamera));
	printf("sSkeleton %lu\n",sizeof(sSkeleton));
	printf("sBone %lu\n",sizeof(sBone));
	printf("sShader %lu\n",sizeof(sShader));
	printf("sMaterial %lu\n",sizeof(sMaterial));
	printf("sTexture %lu\n",sizeof(sTexture));
	printf("fElement %lu\n",sizeof(fElement));
	printf("fForm %lu\n",sizeof(fForm));
}
