/*
 * physics.c
 *
 *  Created on: 16 янв. 2018 г.
 *      Author: ivan
 */

#include "engine.h"

#define get_index_by_name(name,objects,length) _get_index_by_name(objects,sizeof(objects[0]),name,length)

static float _radar_cone_vals[] = {
		  0.0,0.0,0.0,
		  0.000000, -1.000000, 1.000000,
		  0.000000,  0.000000, 0.000000,
		  0.195090, -0.980786, 1.000000,
		  0.382683, -0.923880, 1.000000,
		  0.555570, -0.831470, 1.000000,
		  0.707107, -0.707107, 1.000000,
		  0.831470, -0.555570, 1.000000,
		  0.923880, -0.382684, 1.000000,
		  0.980785, -0.195091, 1.000000,
		  1.000000, -0.000000, 1.000000,
		  0.980785,  0.195090, 1.000000,
		  0.923880,  0.382683, 1.000000,
		  0.831470,  0.555570, 1.000000,
		  0.707107,  0.707106, 1.000000,
		  0.555570,  0.831469, 1.000000,
		  0.382683,  0.923879, 1.000000,
		  0.195090,  0.980785, 1.000000,
		 -0.000000,  1.000000, 1.000000,
		 -0.195091,  0.980785, 1.000000,
		 -0.382684,  0.923879, 1.000000,
		 -0.555571,  0.831469, 1.000000,
		 -0.707107,  0.707106, 1.000000,
		 -0.831470,  0.555569, 1.000000,
		 -0.923880,  0.382682, 1.000000,
		 -0.980785,  0.195089, 1.000000,
		 -1.000000, -0.000001, 1.000000,
		 -0.980785, -0.195092, 1.000000,
		 -0.923879, -0.382685, 1.000000,
		 -0.831469, -0.555572, 1.000000,
		 -0.707106, -0.707108, 1.000000,
		 -0.555569, -0.831471, 1.000000,
		 -0.382682, -0.923880, 1.000000,
		 -0.195089, -0.980786, 1.000000
};

static index_t _radar_cone_inds[] = {
		1, 2, 3,
		 3, 2, 4,
		 4, 2, 5,
		 5, 2, 6,
		 6, 2, 7,
		 7, 2, 8,
		 8, 2, 9,
		 9, 2, 10,
		10, 2, 11,
		11, 2, 12,
		12, 2, 13,
		13, 2, 14,
		14, 2, 15,
		15, 2, 16,
		16, 2, 17,
		17, 2, 18,
		18, 2, 19,
		19, 2, 20,
		20, 2, 21,
		21, 2, 22,
		22, 2, 23,
		23, 2, 24,
		24, 2, 25,
		25, 2, 26,
		26, 2, 27,
		27, 2, 28,
		28, 2, 29,
		29, 2, 30,
		30, 2, 31,
		31, 2, 32,
		32, 2, 33,
		33, 2,  1,
		17, 25, 9,
		 1, 2, 18,
		17, 2, 33,
		16, 2, 32,
		15, 2, 31,
		14, 2, 30,
		13, 2, 29,
		12, 2, 28,
		11, 2, 27,
		10, 2, 26,
		 9, 2, 25,
		 8, 2, 24,
		 7, 2, 23,
		 6, 2, 22,
		 5, 2, 21,
		 4, 2, 20,
		 3, 2, 19,
		33, 1, 3,
		 3, 4, 5,
		 5, 6, 7,
		 7, 8, 5,
		 9, 10, 11,
		11, 12, 9,
		13, 14, 17,
		15, 16, 17,
		17, 18, 21,
		19, 20, 21,
		21, 22, 23,
		23, 24, 25,
		25, 26, 27,
		27, 28, 29,
		29, 30, 33,
		31, 32, 33,
		33, 3, 9,
		 5, 8, 9,
		 9, 12, 13,
		14, 15, 17,
		18, 19, 21,
		21, 23, 25,
		25, 27, 33,
		30, 31, 33,
		 3, 5, 9,
		 9, 13, 17,
		17, 21, 25,
		27, 29, 33,
		33, 9, 25
};

static dGeomID _dCreateCone(dSpaceID space,float length,float angle)
{
	dTriMeshDataID mesh_data = dGeomTriMeshDataCreate();
	dGeomID geom;
	angle/=2.0;
	float *vals = malloc(sizeof(_radar_cone_vals));
	for (index_t i=0;i<sizeof(_radar_cone_vals)/sizeof(_radar_cone_vals[0]);i+=3)
	{
		vals[i+0] = _radar_cone_vals[i+0]*length*tanf(angle);
		vals[i+1] = _radar_cone_vals[i+1]*length*tanf(angle);
		vals[i+2] = _radar_cone_vals[i+2]*length;
	}
	dGeomTriMeshDataBuildSingle(mesh_data,
									vals,12,sizeof(_radar_cone_vals)/12,
									_radar_cone_inds,sizeof(_radar_cone_inds)/sizeof(index_t),sizeof(index_t[3]));
	geom = dCreateTriMesh(space,mesh_data,0,0,0);
	free(vals);
	return geom;
}

static size_s _get_index_by_name(void* objects,size_s size,char* name,size_s length)
{
	uint32_t hash,index=0;
	hash = S_Name2hash(name);
	for (;index<length;index++)
	{
		if (hash==*(uint32_t*)(((char*)objects)+index*size))
			return index;
	}
	return -1;
}

static inline void _read_body_position(sObject* object)
{
	//puts(object->name);
	if (!object->physics_enabled) return;
	laTypeD laRot = laTypeCastToDouble(&object->transform_global);
	dReal *rot = laRot.a;
	if (object->ray.radar_mesh)
	{
		int inv = (object->ray.dir>>2) ? -1 : 1;
		switch (object->ray.dir&3)
		{
			case rayX : dGeomRaySet(object->ray.radar_mesh,rot[3],rot[7],rot[11], rot[0]*inv, rot[4]*inv, rot[8] *inv);break;
			case rayY : dGeomRaySet(object->ray.radar_mesh,rot[3],rot[7],rot[11], rot[1]*inv, rot[5]*inv, rot[9] *inv);break;
			case rayZ : dGeomRaySet(object->ray.radar_mesh,rot[3],rot[7],rot[11], rot[2]*inv, rot[6]*inv, rot[10]*inv);break;
		}
		dGeomSetPosition(object->ray.radar_mesh,rot[3],rot[7],rot[11]);
	}
	if (object->radar.radar_mesh)
	{
		laType rotation = object->transform_global;
		dGeomSetPosition(object->radar.radar_mesh,rotation.a[3],rotation.a[7],rotation.a[11]);

		rotation.a[3] = rotation.a[7] = rotation.a[11] = 0.0;
		switch (object->ray.dir)
		{
		case rayY : RotateXYZlocal(&rotation, 3.1415926535/2,0.0,0.0);break;
		case rayYn: RotateXYZlocal(&rotation,-3.1415926535/2,0.0,0.0);break;
		case rayX : RotateXYZlocal(&rotation, 0.0,  3.1415926535/2,0.0);break;
		case rayXn: RotateXYZlocal(&rotation, 0.0, -3.1415926535/2,0.0);break;
		case rayZn: RotateXYZlocal(&rotation, 3.1415926535, 0.0, 0.0);break;
		case rayZ : break;
		}

		dGeomSetRotation(object->radar.radar_mesh,laTypeCastToDouble(&rotation).a);
	}

	if (!object->geom) return;
	if (object->physicsType==1)
	{
		dGeomSetPosition(object->geom,rot[3],rot[7],rot[11]);
		rot[3] = 0;
		rot[7] = 0;
		rot[11] = 0;
		dGeomSetRotation(object->geom,rot);
		/*if (object->ray.radar_mesh)
		{
			int inv = (object->ray.dir>>2) ? -1 : 1;
			switch (object->ray.dir&3)
			{
				case rayX : dGeomRaySet(object->ray.radar_mesh,rot[3],rot[7],rot[11], rot[0]*inv, rot[4]*inv, rot[8] *inv);break;
				case rayY : dGeomRaySet(object->ray.radar_mesh,rot[3],rot[7],rot[11], rot[1]*inv, rot[5]*inv, rot[9] *inv);break;
				case rayZ : dGeomRaySet(object->ray.radar_mesh,rot[3],rot[7],rot[11], rot[2]*inv, rot[6]*inv, rot[10]*inv);break;
			}
		}*/
	}
	else if (object->physicsType>1)
	{
		if (!object->body) return;

		dReal *r = (dReal*)dBodyGetRotation(object->body);
		object->transform = Matrix3x3(r[0],r[1],r[2],
									  r[4],r[5],r[6],
									  r[8],r[9],r[10]);
		dReal *p = (dReal*)dBodyGetPosition(object->body);
		object->transform.a[3] = p[0];
		object->transform.a[7] = p[1];
		object->transform.a[11]= p[2];
		/*if (object->ray.radar_mesh)
		{
			int inv = (object->ray.dir>>2) ? -1 : 1;
			switch (object->ray.dir&3)
			{
				case rayX : dGeomRaySet(object->ray.radar_mesh,p[0],p[1],p[2], r[0]*inv, r[4]*inv, r[8]*inv);break;
				case rayY : dGeomRaySet(object->ray.radar_mesh,p[0],p[1],p[2], r[1]*inv, r[5]*inv, r[9]*inv);break;
				case rayZ : dGeomRaySet(object->ray.radar_mesh,p[0],p[1],p[2], r[2]*inv, r[6]*inv, r[10]*inv);break;
			}
		}*/
	}
}

static void _collision_callback(void* scn, dGeomID o1, dGeomID o2)
{
	dBodyID b1 = dGeomGetBody(o1);sObject* mesh_1 = dGeomGetData(o1);
	dBodyID b2 = dGeomGetBody(o2);sObject* mesh_2 = dGeomGetData(o2);
	if ((mesh_1->physicsType==1 && mesh_2->physicsType==1)
			|| (o1==o2)
			//|| (b1==b2 && b1!=0 && !(mesh_1->radar.radar_mesh || mesh_2->radar.radar_mesh))
			|| (mesh_1==mesh_2)
			|| !((mesh_1->collisionGroups&mesh_2->collideWithGroups) && (mesh_2->collisionGroups&mesh_1->collideWithGroups))
			|| ((mesh_1->bodyIgnoring==mesh_2->body || mesh_2->bodyIgnoring==mesh_1->body) && dGeomGetClass(o1)!=dRayClass && dGeomGetClass(o2)!=dRayClass && mesh_2->radar.radar_mesh!=o2 && mesh_1->radar.radar_mesh!=o1)
			|| !mesh_1->physics_enabled || !mesh_2->physics_enabled
			) return;
	/*_Bool ret = 1;
	if (b1) ret = ret && !dBodyIsEnabled(b1);
	if (b2) ret = ret && !dBodyIsEnabled(b2);
	if (ret) return;*/
	sScene* scene = scn;
	int contact_count = 16;
	dContact contact[contact_count];
	dMass m1, m2;
	if (b1)
		dBodyGetMass(b1, &m1);
	if (b2)
		dBodyGetMass(b2, &m2);
	for (uint32_t i=0; i<contact_count; i++)
	{
		contact[i].surface.mode = dContactSoftCFM;// | dContactSoftERP;
		//contact[i].surface.mode = dContactSoftERP;// | dContactMu2;
		float fr1 = mesh_1->mesh ? mesh_1->mesh->material->friction : 1;
		float fr2 = mesh_2->mesh ? mesh_2->mesh->material->friction : 1;
		contact[i].surface.mu = sqrtf(fr1*fr2);
		//contact[i].surface.mu2 = mesh_2->mesh->material->friction;
		contact[i].surface.bounce = 0.0001;
		contact[i].surface.bounce_vel = 0.0001;
		contact[i].surface.soft_erp = 0.7;
		contact[i].surface.soft_cfm = 0.001;// / ((b1!=0 ? m1.mass : 1.0) + (b2!=0 ? m2.mass : 1.0))*0.5;
	}
	/*uint64_t B1 = (intptr)b1;
	uint64_t B2 = (intptr)b2;
	B1 *= dGeomGetClass(o1)!=5;
	B2 *= dGeomGetClass(o2)!=5;
	b1 = (dBodyID)B1;
	b2 = (dBodyID)B2;*/

	int numc = dCollide(o1, o2, 16, &contact[0].geom, sizeof(dContact));

	if (numc)
	{
		if (mesh_1->ray.radar_mesh==o1 && mesh_1->ray.contacts)
		{
			sPhysicsRSAddContact(mesh_1,mesh_2,contact[0].geom.pos,contact[0].geom.normal);
			return;
		}
		if (mesh_2->ray.radar_mesh==o2 && mesh_2->ray.contacts)
		{
			sPhysicsRSAddContact(mesh_2,mesh_1,contact[0].geom.pos,contact[0].geom.normal);
			return;
		}
		if (mesh_1->radar.radar_mesh==o1 && mesh_1->radar.contacts)
		{
			sPhysicsRadSAddContact(mesh_1,mesh_2,contact[0].geom.pos,contact[0].geom.normal);
			return;
		}
		if (mesh_2->radar.radar_mesh==o2 && mesh_2->radar.contacts)
		{
			sPhysicsRadSAddContact(mesh_2,mesh_1,contact[0].geom.pos,contact[0].geom.normal);
			return;
		}

		if (dGeomGetClass(o1)!=dRayClass && dGeomGetClass(o2)!=dRayClass && mesh_2->radar.radar_mesh!=o2 && mesh_1->radar.radar_mesh!=o1)
		{
			if (mesh_1->collider.contacts)
			{
				sPhysicsCSAddContact(mesh_1,mesh_2,contact[0].geom.pos,contact[0].geom.normal);
			}
			if (mesh_2->collider.contacts)
			{
				sPhysicsCSAddContact(mesh_2,mesh_1,contact[0].geom.pos,contact[0].geom.normal);
			}
		}
		if (mesh_1->ghost || mesh_2->ghost) return;
		for (uint32_t i=0; i<numc; i++)
		{
			dJointID c = dJointCreateContact(scene->world, scene->contactgroup, contact + i);
			dJointAttach(c, b1, b2);
		}
	}
}

static dGeomID load_triangle_mesh(char* name,sScene* scene)
{
	dTriMeshDataID mesh_data = dGeomTriMeshDataCreate();
	dGeomID geom;
	FILE* fp;
	sMesh* mesh;
	index_t index;

	index = get_index_by_name(name,scene->meshes,scene->mesh_count);
	if (index==(0xFFFFFFFF)) {fprintf(stderr,"Triangle mesh %s does not exist\n",name);exit(-1);};
	mesh = &scene->meshes[index];

	if (!mesh->vertices)
	{
		sprintf(buff,RESDIR"/mesh/%s"RESMESH,name);
		//printf("Open file %s\n",buff);
		fp = fopen(buff,"rb");
		if (!fp)
		{
			fprintf(stderr,"Triangle mesh file %s not found\n",buff);
			exit(-1);
		}
		uint64_t l;
		readf(&l,1,8,fp);
		readf(&mesh->deformed,1,1,fp);
		readf(&mesh->uv2,1,1,fp);
		readf(&mesh->ind_count,1,4,fp);mesh->indices = sCalloc(sizeof(index_t),mesh->ind_count);
		readf(mesh->indices,sizeof(index_t),(size_s)mesh->ind_count,fp);
		readf(&mesh->vert_count,1,4,fp);mesh->vertices = sCalloc(sizeof(smVertex)+mesh->deformed*12+mesh->uv2*8,mesh->vert_count);
		readf(mesh->vertices,sizeof(smVertex)+mesh->deformed*12+mesh->uv2*8,mesh->vert_count,fp);
		fclose(fp);
	}

	dGeomTriMeshDataBuildSingle(mesh_data,
								mesh->vertices,sizeof(smVertex)+mesh->deformed*12+mesh->uv2*8,mesh->vert_count,
								mesh->indices,mesh->ind_count,sizeof(uint32_t[3]));
								//,((void*)mesh->vertices) + 12);
	geom = dCreateTriMesh(scene->space,mesh_data,0,0,0);
	return geom;
}


void sPhysicsAttach(sObject* object)
{
	sScene* scene = object->scene;
	sMesh* mesh = object->mesh;
	dSpaceID space = scene->space;
	if (!object->physicsType) return;
	//printf("Object %s has %s physics\n",object->name,object->physicsType==1 ? "static" : "dynamic");
	dGeomID geom = 0;
	switch (object->physicsShape)
	{
	case 0 : geom = dCreateCapsule(space,mesh->bounding_box.a[0]/2.0,mesh->bounding_box.a[2]-mesh->bounding_box.a[0]);break;
	case 1 : geom = dCreateBox(space,mesh->bounding_box.a[0],mesh->bounding_box.a[1],mesh->bounding_box.a[2]);break;
	case 2 : geom = dCreateSphere(space,mesh->bounding_box.a[0]/2.0);break;
	case 3 : geom = dCreateCylinder(space,mesh->bounding_box.a[0]/2.0,mesh->bounding_box.a[2]);break;
	default : geom = load_triangle_mesh(mesh->name,scene);break;
	}
	object->collisionGroups = 0xFFFFFFFF;
	object->collideWithGroups = 0xFFFFFFFF;
	object->geom = geom;
	object->bodyIgnoring = (void*)-1l;
	object->physics_enabled = 1;
	dReal rotation[] = {object->transform.a[0],object->transform.a[1],object->transform.a[2],0.0,
                            object->transform.a[4],object->transform.a[5],object->transform.a[6],0.0,
                            object->transform.a[8],object->transform.a[9],object->transform.a[10],0.0};
	if (object->physicsType == 1)
	{
            dGeomSetPosition(geom,object->transform.a[3],object->transform.a[7],object->transform.a[11]);
            dGeomSetRotation(geom,rotation);
            dGeomSetData(geom,object);
	}
	else if (object->physicsType == 2 || object->physicsType == 3)
	{
            dMass mass;
            memset(&mass,0,sizeof(dMass));
            object->body = dBodyCreate(scene->world);
            dGeomSetBody(geom,object->body);
            //printf("Mass of %s %.5lf\n",object->name,object->physicsMass);
            if (object->physicsType == 2)
            {
                switch (object->physicsShape)
                {
                    case 0 : dMassSetCapsuleTotal(&mass,object->physicsMass,2,mesh->bounding_box.a[0]/2,mesh->bounding_box.a[2]/2);break;
                    default : dMassSetBoxTotal(&mass,object->physicsMass,mesh->bounding_box.a[0],mesh->bounding_box.a[1],mesh->bounding_box.a[2]);break;
                    case 2 : dMassSetSphereTotal(&mass,object->physicsMass,mesh->bounding_box.a[0]/2);break;
                    case 3 : dMassSetCylinderTotal(&mass,object->physicsMass,2,mesh->bounding_box.a[0]/2,mesh->bounding_box.a[2]/2);break;
                }
            }
            else
            {
                dMassSetBoxTotal(&mass,object->physicsMass,99999999,99999999,99999999);
            }
            dBodySetMass(object->body,&mass);
            dBodySetRotation(object->body,rotation);
            dBodySetPosition(object->body,object->transform.a[3],object->transform.a[7],object->transform.a[11]);
            dBodySetAutoDisableFlag(object->body,1);
            dBodySetAutoDisableLinearThreshold(object->body,0.1);
            dBodySetAutoDisableSteps(object->body,10);

            //dBodySetMaxAngularSpeed(object->body,100.0);
            dBodySetDamping(object->body,0.001,0.001);
            //dBodySetLinearDampingThreshold(object->body,0.01);
            //dBodySetAngularDampingThreshold(object->body,0.01);

            dBodySetData(object->body,object);
            dGeomSetData(geom,object);
	}
}

void sPhysicsSetAngularVelocity(sObject* object, double x, double y, double z)
{
	if (object->name[0]!='o') return;
	if (object->body)
	{
		dBodySetAngularVel(object->body, x,y,z);
	}
}

void sPhysicsApplyForceAtPointGlobal3fv(sObject* obj,laType pos,laType vec)
{
	if (obj->body)
	{
		dBodyEnable(obj->body);
		dBodyAddForceAtPos(obj->body,vec.a[0],vec.a[1],vec.a[2],pos.a[0],pos.a[1],pos.a[2]);
	}
}

void sPhysicsApplyImpulseAtPointGlobal3fv(sObject* obj,laType pos,laType vec)
{
	if (obj->body)
	{
		vec = Mulf(vec,obj->physicsMass);
		dBodyEnable(obj->body);
		dBodyAddForceAtPos(obj->body,vec.a[0],vec.a[1],vec.a[2],pos.a[0],pos.a[1],pos.a[2]);
	}
}

void sPhysicsApplyHitAtPointGlobal3fv(sObject* obj,laType pos,laType vec,float mass)
{
	if (obj->body)
	{
		vec = Mulf(vec,mass);
		vec = Divf(vec,mass+obj->physicsMass);
		dBodyEnable(obj->body);
		dBodyAddForceAtPos(obj->body,vec.a[0],vec.a[1],vec.a[2],pos.a[0],pos.a[1],pos.a[2]);
	}
}

void sPhysicsInit(sScene* scene)
{
	dInitODE2(0);
	scene->world = dWorldCreate();
	scene->space = dHashSpaceCreate(0);
	scene->contactgroup = dJointGroupCreate(0);
	scene->joints = dJointGroupCreate(0);
	dWorldSetGravity(scene->world, 0.0,0.0,-9.8);

	dWorldSetERP(scene->world,0.8);
	dWorldSetCFM(scene->world,0.00000001);
    dWorldSetQuickStepNumIterations(scene->world,200);
    int _min,_max;
    dHashSpaceGetLevels(scene->space,&_min,&_max);
    //printf("hash min %d, max %d\n",_min,_max);
    dHashSpaceSetLevels(scene->space,_min,320);
}

void sPhysicsStep(sScene* scene,double step)
{
	for (index_t i=0;i<scene->objects_count;i++)
	{
		sPhysicsCSClear(scene->objects[i]);
		sPhysicsRSClear(scene->objects[i]);
		sPhysicsRadSClear(scene->objects[i]);
		_read_body_position(scene->objects[i]);
	}

	dSpaceCollide(scene->space,scene,_collision_callback);
	dWorldQuickStep(scene->world, step);
	dJointGroupEmpty(scene->contactgroup);
}

void sPhysicsStop(sScene* scene)
{
	dJointGroupDestroy(scene->contactgroup);
	dJointGroupDestroy(scene->joints);
	dSpaceDestroy(scene->space);
	dWorldDestroy(scene->world);
	dCloseODE();
}

void sPhysicsSetSpeedGlobal(void* object,laType vel,uint8_t axes)
{
	sObject* obj = object;
	if (!obj->physicsType) return;
	const dReal* old_vel = dBodyGetLinearVel(obj->body);
	dBodySetLinearVel(obj->body,(axes&1) ? vel.a[0] : old_vel[0],(axes&2) ? vel.a[1] : old_vel[1],(axes&4) ? vel.a[2] : old_vel[2]);
}

void sPhysicsSetSpeedXLocal(void* object,float vel)
{
	sObject* obj = object;
	if (!obj->physicsType) return;
	const dReal* old_vel = dBodyGetLinearVel(obj->body);
	dReal new_vel[3] = {old_vel[0],old_vel[1],old_vel[2]};
	float angle = Dot(Vector(obj->transform_global.a[0],obj->transform_global.a[4],obj->transform_global.a[8]),Vector(new_vel[0],new_vel[1],new_vel[2]));
	if (angle==NAN) return;

	new_vel[0]+= obj->transform_global.a[0]*(vel-angle);
	new_vel[1]+= obj->transform_global.a[4]*(vel-angle);
	new_vel[2]+= obj->transform_global.a[8]*(vel-angle);

	dBodySetLinearVel(obj->body,new_vel[0],new_vel[1],new_vel[2]);
}

void sPhysicsSetSpeedYLocal(void* object,float vel)
{
	sObject* obj = object;
	if (!obj->physicsType) return;
	const dReal* old_vel = dBodyGetLinearVel(obj->body);
	dReal new_vel[3] = {old_vel[0],old_vel[1],old_vel[2]};
	float angle = Dot(Vector(obj->transform_global.a[1],obj->transform_global.a[5],obj->transform_global.a[9]),Vector(new_vel[0],new_vel[1],new_vel[2]));
	if (angle==NAN) return;

	new_vel[0]+= obj->transform_global.a[1]*(vel-angle);
	new_vel[1]+= obj->transform_global.a[5]*(vel-angle);
	new_vel[2]+= obj->transform_global.a[9]*(vel-angle);

	dBodySetLinearVel(obj->body,new_vel[0],new_vel[1],new_vel[2]);
}

void sPhysicsSetSpeedZLocal(void* object,float vel)
{
	sObject* obj = object;
	if (!obj->physicsType) return;
	const dReal* old_vel = dBodyGetLinearVel(obj->body);
	dReal new_vel[3] = {old_vel[0],old_vel[1],old_vel[2]};
	float angle = Dot(Vector(obj->transform_global.a[2],obj->transform_global.a[6],obj->transform_global.a[10]),Vector(new_vel[0],new_vel[1],new_vel[2]));
	if (angle==NAN) return;

	new_vel[0]+= obj->transform_global.a[2]*(vel-angle);
	new_vel[1]+= obj->transform_global.a[6]*(vel-angle);
	new_vel[2]+= obj->transform_global.a[10]*(vel-angle);

	dBodySetLinearVel(obj->body,new_vel[0],new_vel[1],new_vel[2]);
}
void sPhysicsCSInit(sObject* object)
{
	sPhysicsCSFree(object);
	sPhysicsCS* sensor = &object->collider;
	sensor->space = ((sScene*)object->scene)->space;
	sensor->contactsCount = 0;
	sensor->contactsAllocated = 2;
	sensor->contacts = sMalloc(sizeof(sPhysicsContact)*sensor->contactsAllocated);
}
void sPhysicsRSInit(sObject* object,float range)
{
	sPhysicsRSFree(object);
	sPhysicsRS* sensor = &object->ray;
	sensor->space = ((sScene*)object->scene)->space;
	sensor->contactsCount = 0;
	sensor->contactsAllocated = 2;
	sensor->contacts = sMalloc(sizeof(sPhysicsContact)*sensor->contactsAllocated);

	sensor->radar_mesh = dCreateRay(((sScene*)object->scene)->space,range);
	dGeomRaySetClosestHit(sensor->radar_mesh,1);
	//dGeomRaySetBackfaceCull(sensor->radar_mesh,0);
	dGeomSetData(sensor->radar_mesh,object);
}

void sPhysicsRadSInit(sObject* object,float range,float angle)
{
	sPhysicsRadSFree(object);
	sPhysicsRS* sensor = &object->radar;
	sensor->space = ((sScene*)object->scene)->space;
	sensor->contactsCount = 0;
	sensor->contactsAllocated = 2;
	sensor->contacts = sMalloc(sizeof(sPhysicsContact)*sensor->contactsAllocated);
	sensor->angle = cosf(angle);
	sensor->radar_mesh = _dCreateCone(((sScene*)object->scene)->space,range,angle);
	dGeomSetData(sensor->radar_mesh,object);
}

void sPhysicsCSClear(sObject* object)
{
	//printf("CS Clear\n");
	sPhysicsCS* sensor = &object->collider;
	if (sensor->contacts && sensor->contactsAllocated-sensor->contactsCount>sensor->contactsAllocated/2 && sensor->contactsCount>2)
	{
		sensor->contactsAllocated = sensor->contactsCount;
		sensor->contacts = sRealloc(sensor->contacts,sizeof(sPhysicsContact)*sensor->contactsAllocated);
	}
	sensor->contactsCount = 0;
	//printf("CS Cleared\n");
}
void sPhysicsRSClear(sObject* object)
{
	//printf("RS Clear\n");
	sPhysicsRS* sensor = &object->ray;
	if (sensor->contacts && sensor->contactsAllocated-sensor->contactsCount>sensor->contactsAllocated/2 && sensor->contactsCount>2)
	{
		sensor->contactsAllocated = sensor->contactsCount;
		sensor->contacts = sRealloc(sensor->contacts,sizeof(sPhysicsContact)*sensor->contactsAllocated);
	}
	sensor->contactsCount = 0;
	//printf("RS Cleared\n");
}
void sPhysicsRadSClear(sObject* object)
{
	//printf("RS Clear\n");
	sPhysicsRS* sensor = &object->radar;
	if (sensor->contacts && sensor->contactsAllocated-sensor->contactsCount>sensor->contactsAllocated/2 && sensor->contactsCount>2)
	{
		sensor->contactsAllocated = sensor->contactsCount;
		sensor->contacts = sRealloc(sensor->contacts,sizeof(sPhysicsContact)*sensor->contactsAllocated);
	}
	sensor->contactsCount = 0;
}

void sPhysicsCSFree(sObject* object)
{
	sPhysicsCS* sensor = &object->collider;
	if (!sensor) return;
	sensor->contactsCount = 0;
	sensor->contactsAllocated = 0;
	if (sensor->contacts)
	{
		sFree(sensor->contacts);
		sensor->contacts = 0;
	}
	sensor->contacts = 0;
}
void sPhysicsRSFree(sObject* object)
{
	sPhysicsRS* sensor = &object->ray;
	sensor->contactsCount = 0;
	sensor->contactsAllocated = 0;
	if (sensor->contacts)
	{
		sFree(sensor->contacts);
		sensor->contacts = 0;
	}
	sensor->contacts = 0;
	if (sensor->radar_mesh)
	{
		dGeomSetData(sensor->radar_mesh,0);
		dGeomDestroy(sensor->radar_mesh);
		sensor->radar_mesh = 0;
	}
}
void sPhysicsRadSFree(sObject* object)
{
	sPhysicsRS* sensor = &object->radar;
	sensor->contactsCount = 0;
	sensor->contactsAllocated = 0;
	if (sensor->contacts)
	{
		sFree(sensor->contacts);
		sensor->contacts = 0;
	}
	sensor->contacts = 0;
	if (sensor->radar_mesh)
	{
		dGeomSetData(sensor->radar_mesh,0);
		dGeomDestroy(sensor->radar_mesh);
		sensor->radar_mesh = 0;
	}
}

void sPhysicsCSAddContact(sObject* object,sObject* obj,dReal* position,dReal* normal)
{
	sPhysicsCS* sensor = &object->collider;
	if (!sensor->contacts) return;
	//printf("sPhysicsCSAddContact %d\n",sensor->contactsCount);
	if (sensor->contactsCount>=sensor->contactsAllocated)
	{
		sensor->contactsAllocated *= 2;
		//printf("sPhysicsCSExpand %d %ld\n",sensor->contactsAllocated,sizeof(sPhysicsContact)*sensor->contactsAllocated);
		sensor->contacts = sRealloc(sensor->contacts,sizeof(sPhysicsContact)*sensor->contactsAllocated);
	}
	//printf("sPhysicsCSCopying\n");
	memcpy(sensor->contacts[sensor->contactsCount].position,position,sizeof(dReal[3]));
	memcpy(sensor->contacts[sensor->contactsCount].normal,normal,sizeof(dReal[3]));
	sensor->contacts[sensor->contactsCount++].object = obj;
	//printf("sPhysicsCSAdded\n");
}
void sPhysicsRSAddContact(sObject* object,sObject* obj,dReal* position,dReal* normal)
{
	//printf("sPhysicsRSAddContact\n");
	sPhysicsRS* sensor = &object->ray;
	if (!sensor->contacts) return;
	if (sensor->contactsCount>=sensor->contactsAllocated)
	{
		sensor->contactsAllocated *= 2;
		//printf("sPhysicsRSExpand %d %ld\n",sensor->contactsAllocated,sizeof(sPhysicsContact)*sensor->contactsAllocated);
		sensor->contacts = sRealloc(sensor->contacts,sizeof(sPhysicsContact)*sensor->contactsAllocated);
	}

	memcpy(sensor->contacts[sensor->contactsCount].position,position,sizeof(dReal[3]));
	memcpy(sensor->contacts[sensor->contactsCount].normal,normal,sizeof(dReal[3]));
	sensor->contacts[sensor->contactsCount++].object = obj;
	//printf("sPhysicsRSAddContact end\n");
}
void sPhysicsRadSAddContact(sObject* object,sObject* obj,dReal* position,dReal* normal)
{
	sPhysicsRS* sensor = &object->radar;

	if (!sensor->contacts) return;
	if (sensor->contactsCount>=sensor->contactsAllocated)
	{
		sensor->contactsAllocated *= 2;
		sensor->contacts = sRealloc(sensor->contacts,sizeof(sPhysicsContact)*sensor->contactsAllocated);
	}

	memcpy(sensor->contacts[sensor->contactsCount].position,position,sizeof(dReal[3]));
	memcpy(sensor->contacts[sensor->contactsCount].normal,normal,sizeof(dReal[3]));
	sensor->contacts[sensor->contactsCount].object = obj;
	sensor->contactsCount++;
}

void sPhysicsAutoDisable(sObject* obj,uint8_t flag)
{
	if (obj->body)
	{
		dBodySetAutoDisableFlag(obj->body,(_Bool)flag);
	}
}

laType sPhysicsRSGetHitNormal(sObject* obj)
{
	if (obj->ray.radar_mesh && obj->ray.contactsCount)
		return Vector(obj->ray.contacts->normal[0],obj->ray.contacts->normal[1],obj->ray.contacts->normal[2]);
	else
		return Vector(0,0,0);
}
laType sPhysicsRSGetHitPosition(sObject* obj)
{
	if (obj->ray.radar_mesh && obj->ray.contactsCount)
		return Vector(obj->ray.contacts->position[0],obj->ray.contacts->position[1],obj->ray.contacts->position[2]);
	else
		return Vector(0,0,0);
}

/*void sPhysicsRSSetDirection(sObject* obj,int dir)
{
	obj->ray.dir = dir;
}*/
void sPhysicsRSSetRange(sPhysicsRS* ray,float range)
{
	if (dGeomGetClass(ray->radar_mesh) == dRayClass)
		dGeomRaySetLength(ray->radar_mesh,range);
	else if (dGeomGetClass(ray->radar_mesh) == dSphereClass)
		dGeomSphereSetRadius(ray->radar_mesh,range);
	else if (dGeomGetClass(ray->radar_mesh) == dBoxClass)
		dGeomBoxSetLengths(ray->radar_mesh,range,range,range);
	else if (dGeomGetClass(ray->radar_mesh) == dTriMeshClass)
	{
		dGeomDestroy(ray->radar_mesh);
		ray->radar_mesh = _dCreateCone(ray->space,ray->range,ray->angle);
	}
}
void sPhysicsRadarSetAngle(sPhysicsRS* radar,float angle)
{
	radar->angle = cosf(angle);
	dGeomDestroy(radar->radar_mesh);
	radar->radar_mesh = _dCreateCone(radar->space,radar->range,radar->angle);
}

/*void sPhysicsRadSSetRange(sPhysicsRS* radar,float range)
{
	dGeomSphereSetRadius(radar->radar_mesh,range);
}*/
void sPhysicsRadSSetDirection(sPhysicsRS* radar,int dir)
{
	radar->dir = dir;
}
/*void sPhysicsRadSSetAngle(sPhysicsRS* radar,float angle)
{
	radar->angle = cosf(angle);
}*/

laType sPhysicsGetLinearVelocity(sObject* obj)
{
	if (obj->body)
	{
		const dReal* vel = dBodyGetLinearVel(obj->body);
		return Vector(vel[0],vel[1],vel[2]);
	}
	return Vector(0.0, 0.0, 0.0);
}

laType sPhysicsRSGetHitNormal3f(sObject* obj,index_t ind)
{
	if (obj->ray.radar_mesh && obj->ray.contactsCount)
	{
		return Vector(obj->ray.contacts[ind].normal[0],obj->ray.contacts[ind].normal[1],obj->ray.contacts[ind].normal[2]);
	}
	else
	{
		return Vector(0.0f, 0.0f, 0.0f);
	}
}
laType sPhysicsRSGetHitPosition3f(sObject* obj,index_t ind)
{
	if (obj->ray.radar_mesh && obj->ray.contactsCount)
	{
		return Vector(obj->ray.contacts[ind].position[0],obj->ray.contacts[ind].position[1],obj->ray.contacts[ind].position[2]);
	}
	else
	{
		return Vector(0.0f, 0.0f, 0.0f);
	}
}


laType sPhysicsRadSGetHitNormal3f(sObject* obj,index_t ind)
{
	if (obj->radar.radar_mesh && obj->radar.contactsCount)
	{
		return Vector(obj->radar.contacts[ind].normal[0],obj->radar.contacts[ind].normal[1],obj->radar.contacts[ind].normal[2]);
	}
	else
	{
		return Vector(0.0,0.0,0.0);
	}
}
laType sPhysicsRadSGetHitPosition3f(sObject* obj,index_t ind)
{
	if (obj->radar.radar_mesh && obj->radar.contactsCount)
	{
		return Vector(obj->radar.contacts[ind].position[0],obj->radar.contacts[ind].position[1],obj->radar.contacts[ind].position[2]);

	}
	else
	{
		return Vector(0.0,0.0,0.0);
	}
}

laType sPhysicsCSGetHitNormal3f(sObject* obj,index_t ind)
{
	if (obj->radar.radar_mesh && obj->radar.contactsCount)
	{
		return Vector(obj->radar.contacts[ind].normal[0],obj->radar.contacts[ind].normal[1],obj->radar.contacts[ind].normal[2]);
	}
	else
	{
		return Vector(0.0,0.0,0.0);
	}
}
laType sPhysicsCSGetHitPosition3f(sObject* obj,index_t ind)
{
	if (obj->collider.contactsCount)
	{
		return Vector(obj->collider.contacts[ind].position[0],obj->collider.contacts[ind].position[1],obj->collider.contacts[ind].position[2]);
	}
	else
	{
		return Vector(0.0f, 0.0f, 0.0f);
	}
}

index_t sPhysicsCSGetHitObjectCount(sObject* obj)
{
	return obj->collider.contactsCount;
}
sObject* sPhysicsCSGetHitObject(sObject* obj,index_t ind)
{
	return obj->collider.contacts[ind].object;
}

index_t sPhysicsRadSGetHitObjectCount(sObject* obj)
{
	return obj->radar.contactsCount;
}
sObject* sPhysicsRadSGetHitObject(sObject* obj,index_t ind)
{
	return obj->radar.contacts[ind].object;
}

index_t sPhysicsRSGetHitObjectCount(sObject* obj)
{
	return obj->ray.contactsCount;
}
sObject* sPhysicsRSGetHitObject(sObject* obj,index_t ind)
{
	return obj->ray.contacts[ind].object;
}

void sPhysicsSuspend(sObject* object)
{
	dBodyDisable(object->body);
	object->physics_enabled = 0;
}

void sPhysicsResume(sObject* object)
{
	dBodyEnable(object->body);
	object->physics_enabled = 1;
	dReal rotation[] = {object->transform.a[0],object->transform.a[1],object->transform.a[2],0.0,
						object->transform.a[4],object->transform.a[5],object->transform.a[6],0.0,
						object->transform.a[8],object->transform.a[9],object->transform.a[10],0.0};

    dBodySetRotation(object->body, rotation);
    dBodySetPosition(object->body, object->transform.a[3], object->transform.a[7], object->transform.a[11]);
}

dJointID sPhysicsCreateAnchor(sObject* obj1,sObject* obj2,dReal minAngle,dReal maxAngle,laType axis_pos_relative,laType axis_dir, _Bool relative)
{
	if (obj1->scene!=obj2->scene || axis_pos_relative.type!=VECTOR || axis_dir.type!=VECTOR) return 0;
	sScene* scene = obj1->scene;
	dJointGroupID jointGroup = scene->joints;
	if (maxAngle<minAngle)
	{
		dReal w=minAngle;
		minAngle=maxAngle;
		maxAngle=w;
	}
	dJointID result = dJointCreateHinge(scene->world,jointGroup);
	dJointAttach(result,obj1->body,obj2->body);
	dJointSetHingeAxis(result,axis_dir.a[0],axis_dir.a[1],axis_dir.a[2]);
	dJointSetHingeAnchor(result,obj1->transform_global.a[3]*relative + axis_pos_relative.a[0],
								obj1->transform_global.a[7]*relative + axis_pos_relative.a[1],
								obj1->transform_global.a[11]*relative+ axis_pos_relative.a[2]);
	if (maxAngle-minAngle > 2*3.1415926535) return result;
	if (maxAngle> 3.14) maxAngle= 3.14;
	if (minAngle<-3.14) minAngle=-3.14;
	dJointSetHingeParam(result,dParamLoStop, minAngle);
	dJointSetHingeParam(result,dParamHiStop, maxAngle);

	return result;
}

dJointID sPhysicsCreateBallSocket(sObject* obj1,sObject* obj2,laType anch_pos_relative, _Bool relative)
{
	if (obj1->scene!=obj2->scene) return 0;
	sScene* scene = obj1->scene;
	dJointGroupID jointGroup = scene->joints;
	dJointID result = dJointCreateBall(scene->world,jointGroup);
	dJointAttach(result,obj1->body,obj2->body);
	dJointSetBallAnchor(result,obj1->transform_global.a[3]*relative + anch_pos_relative.a[0],
								obj1->transform_global.a[7]*relative + anch_pos_relative.a[1],
								obj1->transform_global.a[11]*relative+ anch_pos_relative.a[2]);
	return result;
}

dJointID sPhysicsCreateCardan(sObject* obj1,sObject* obj2,laType anch_pos_relative,laType axis1,laType axis2, _Bool relative)
{
	if (obj1->scene!=obj2->scene) return 0;
	sScene* scene = obj1->scene;
	dJointGroupID jointGroup = scene->joints;
	Normalize(&axis1);
	Normalize(&axis2);
	laType axisn = Crossn(axis1,axis2);
	axis2 = Crossn(axisn,axis1);
	dJointID result = dJointCreateUniversal(scene->world,jointGroup);
	dJointAttach(result,obj1->body,obj2->body);
	dJointSetUniversalAnchor(result,obj1->transform_global.a[3]*relative + anch_pos_relative.a[0],
									obj1->transform_global.a[7]*relative + anch_pos_relative.a[1],
									obj1->transform_global.a[11]*relative+ anch_pos_relative.a[2]);
	dJointSetUniversalAxis1(result,axis1.a[0],axis1.a[1],axis1.a[2]);
	dJointSetUniversalAxis2(result,axis2.a[0],axis2.a[1],axis2.a[2]);
	return result;
}

double sPhysicsJointGetAngle1(dJointID joint)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
	case dJointTypeHinge  : return dJointGetHingeAngle(joint);
	case dJointTypeHinge2 : return dJointGetHinge2Angle1(joint);
	case dJointTypeUniversal : return dJointGetUniversalAngle1(joint);
	default : return 0.0;
	}
}

double sPhysicsJointGetAngle2(dJointID joint)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
	case dJointTypeHinge2 : return dJointGetHinge2Angle2(joint);
	case dJointTypeUniversal : return dJointGetUniversalAngle2(joint);
	default : return 0.0;
	}
}

double sPhysicsJointGetAngle1Rate(dJointID joint)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
	case dJointTypeHinge  : return dJointGetHingeAngleRate(joint);
	case dJointTypeHinge2 : return dJointGetHinge2Angle1Rate(joint);
	case dJointTypeUniversal : return dJointGetUniversalAngle1Rate(joint);
	default : return 0.0;
	}
}

void sPhysicsJointSetAngle1Rate(dJointID joint, double vel, double force)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
	case dJointTypeHinge  :
		dJointSetHingeParam(joint, dParamVel,   vel);
		dJointSetHingeParam(joint, dParamFMax,  force);
		break;
	case dJointTypeHinge2 :
		dJointSetHinge2Param(joint, dParamVel,   vel);
		dJointSetHinge2Param(joint, dParamFMax,  force);
		break;
	case dJointTypeUniversal :
		dJointSetUniversalParam(joint, dParamVel,   vel);
		dJointSetUniversalParam(joint, dParamFMax,  force);
		break;
	}
}

double sPhysicsJointGetAngle2Rate(dJointID joint)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
	case dJointTypeHinge2 : return dJointGetHinge2Angle2Rate(joint);
	case dJointTypeUniversal : return dJointGetUniversalAngle2Rate(joint);
	default : return 0.0;
	}
}

void sPhysicsJointSetAngle2Rate(dJointID joint, double vel, double force)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
	case dJointTypeHinge2 :
		dJointSetHinge2Param(joint, dParamVel2,   vel);
		dJointSetHinge2Param(joint, dParamFMax2,  force);
		break;
	case dJointTypeUniversal :
		dJointSetUniversalParam(joint, dParamVel2,   vel);
		dJointSetUniversalParam(joint, dParamFMax2,  force);
		break;
	}
}

int sPhysicsJointGetAxisCount(dJointID joint)
{
	uint32_t type = dJointGetType(joint);

	switch (type)
	{
		case dJointTypeHinge2 : return 2;
		case dJointTypeUniversal : return 2;
	}
	return 1;
}
