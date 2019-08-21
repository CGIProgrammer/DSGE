/*
 * game_structures.h
 *
 *  Created on: 13 февр. 2018 г.
 *      Author: ivan
 */

#ifndef GAME_STRUCTURES_H_
#define GAME_STRUCTURES_H_


typedef struct
{
	sObject* head;
	sObject* spine1;
	sObject* spine2;
	sObject* spine3;
	sObject* lShoulder;
	sObject* lForearm;
	sObject* rShoulder;
	sObject* rForearm;
	sObject* lLeg;
	sObject* lKnee;
	sObject* rLeg;
	sObject* rKnee;
	sObject* lFoot;
	sObject* rFoot;
	dJointID joints[16];
	dJointGroupID jointGroup;
} sRagdoll;
typedef struct
{
	sObject* body;
	sObject* flw;
	sObject* frw;
	sObject* blw;
	sObject* brw;
	sObject* fls;
	sObject* frs;
	sObject* bls;
	sObject* brs;
	float max_speed;
	float max_torque;
	dJointID joints[8];
	dJointGroupID jointGroup;
	char drive_wheels;
	float spring_damping;
	float spring_force;
	float rpm;
	float acceleration;
	char transmission;
	float power;
	int breaks;
	float gas;
} sVehicle4Wheel;

extern sVehicle4Wheel *sVehicleActive;

void sVehicleInit(sScene* scene,sVehicle4Wheel* vehicle,char* prefix);
void sVehicleTurn(sVehicle4Wheel* vehicle,float amount,uint8_t wheels);
void sVehicleThrottle(sVehicle4Wheel* vehicle);

void sVehicleEngine(sVehicle4Wheel* vehicle);

void sVehicleRelease(sVehicle4Wheel* vehicle);
void sVehicleSetSuspension(sVehicle4Wheel* vehicle,float spring,float dump,uint8_t wheels);
void sVehicleSetMaxSpeedKPH(sVehicle4Wheel* vehicle,float spring);
void sVehicleSetMaxTorque(sVehicle4Wheel* vehicle,float spring);
void sVehicleSetTireFriction(sVehicle4Wheel* vehicle,float friction);

void sRacerInit(sScene *scene,sVehicle4Wheel *vehicle);

sObject* sCharacterInit(sScene *scene,sSkeleton *skeleton, char *name);
sObject* sMobInit(sScene *scene,sSkeleton *skin, char *name, laType bbox);

void sPlayerInit(sScene *scene,sSkeleton *skeleton);
void sPlayerSetImpact(float x,float y,float z);

int sRagdollInit(sScene* scene,sRagdoll* ragdoll, _Bool autodetect,char* prefix);

void sPrintSizeOfAllGameStrictures(void);
void sPlayerMouseLookOn(sScene*);
void sPlayerMouseLookOff(sScene*);

// Искусственный интеллект

typedef struct
{
	index_t count;
	index_t *nodes;
	float *weights;
} sAINode;

typedef struct
{
	index_t node_count;
	sAINode *nodes;
	index_t indices_count;
	index_t *node_indices;
	float *weights;
} sAIMap;

index_t sListElementInList(void *array,unsigned length,void *data,unsigned data_size);
sAIMap sAIGenerateGrid(sMesh *AIMap);

#endif /* GAME_STRUCTURES_H_ */
