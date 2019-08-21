/*
 * objects.c
 *
 *  Created on: 25 дек. 2017 г.
 *      Author: ivan
 */

#include "engine.h"
#include "linalg.h"


laMatrix sObjectGetTransform(sObject* object)
{
    return *(laMatrix*)&object->transform;
}

char* sObjectGetName(sObject* object)
{
    return object->name;
}
char* sMeshGetName(sMesh* mesh)
{
    return mesh->name;
}

void sMeshSetMaterial(sMesh* mesh, sScene* scene, char* name)
{
	mesh->material = sSceneGetMaterial(scene, name);
}

sMesh* sSceneGetMesh(sScene* scene,char* name)
{
    for (int ind=0;ind<scene->mesh_count;ind++)
    {
    	if (!strcmp(name, scene->meshes[ind].name))
    	{
    		return scene->meshes + ind;
    	}
    }
    return 0;
}

sMesh* sObjectGetMesh(sObject* object)
{
    if (object->name[0]!='o')
    {
        fprintf(stderr,"Warning: %s does not have mesh\n",object->name);
        return 0;
    }
    else
        return object->mesh;
}

sTexture* sObjectGetCubemap(sObject* obj)
{
	if (obj && obj->mesh && obj->mesh->material && obj->mesh->material->reflection_cubemap)
	{
		return obj->mesh->material->reflection_cubemap;
	}
	else
	{
		return ((sScene*)obj->scene)->cubemap;
	}
}

void sObjectSetCubemap(sObject* obj, sTexture* cubemap)
{
	if (obj && obj->mesh && obj->mesh->material && obj->mesh->material->reflection_cubemap)
	{
		obj->mesh->material->reflection_cubemap = cubemap;
	}
}

void sObjectSetMesh(sObject* object,sMesh* mesh)
{
    if (object->name[0]=='l')
        fprintf(stderr,"Warning: object of type sLight does not support mesh attaching\n");
    else if (object->name[0]=='s')
        fprintf(stderr,"Warning: object of type sSkeleton does not support mesh attaching\n");
    else if (object->name[0]=='b')
        fprintf(stderr,"Warning: object of type sBone does not support mesh attaching\n");
    else
        object->mesh = mesh;
}
void sObjectSetMeshByName(sObject* object,char* mesh)
{
    sMesh* m = sSceneGetMesh(object->scene,mesh);
    if (m)
        sObjectSetMesh(object,m);
    else
        fprintf(stderr,"Warning: %s no such mesh\n",mesh);
}

sSkeleton* sSkeleton_cast(void* obj)
{
    return (sSkeleton*)obj;
}
sObject* sObject_cast(void* obj)
{
    return (sObject*)obj;
}

sLight* sLight_cast(void* obj)
{
    return (sLight*)obj;
}

sBone* sBone_cast(void* obj)
{
    return (sBone*)obj;
}

sCamera* sCamera_cast(sObject* obj)
{
    return (sCamera*)obj;
}


static void add_child(void* obj,void* child)
{
    sObject* children = child;
    sObject* object = obj;
    //printf("Attach %s to %s\n",children->name,object->name);
    void** old_list = object->children;
    void** new_list = sCalloc(sizeof(void*),object->child_count+1);
    if (old_list)
    {
        memmove(new_list,old_list,object->child_count*sizeof(void*));
        sFree(old_list);
    }
    object->children = new_list;
    object->children[object->child_count++] = child;
    children->parent = object;
}

static void remove_child(void* obj,void* child,_Bool with_parent)
{
    sObject* object = obj;
    index_t ind;
    for (ind=0;ind<object->child_count;ind++)
    {
        if (object->children[ind]==child)
        {
        	memmove(&object->children[ind],&object->children[ind+1],(object->child_count-ind-1)*sizeof(void*));
            if (with_parent)
                ((sObject*)child)->parent = 0;
            object->child_count--;
            if (!object->child_count)
            {
                sFree(object->children);
                object->children = 0;
            }
            break;
        }
    }
}

GameObject* duplicate_object(GameObject* obj)
{
    GameObject* new_object = obj;
    sObject *new_child,*object=(sObject*)obj,*old_child;
    sScene* scene = obj->object.scene;

    if (obj->object.name[0]=='b') // || obj->object.name[0]=='c')
    {
        return obj;
    }
    else if (obj->object.name[0]=='o')
    {
    	new_object = sCalloc(sizeof(sObject),1);
    	memcpy(new_object, obj, sizeof(sObject));
    }
    else if (obj->object.name[0]=='l')
    {
    	new_object = sCalloc(sizeof(sLight),1);
    	memcpy(new_object, obj, sizeof(sLight));
    }
    else if (obj->object.name[0]=='s')
    {
    	uint32_t bonesArraySize = sizeof(sBone) * obj->skeleton.bone_count;
    	uint32_t bonesPoseSize = sizeof(laType) * obj->skeleton.bone_count;
    	new_object = sCalloc(sizeof(sSkeleton),1);
    	memcpy(new_object, obj, sizeof(sSkeleton));
    	new_object->skeleton.bones = sCalloc(sizeof(sBone),  new_object->skeleton.bone_count);
    	new_object->skeleton.pose = sCalloc(sizeof(laType), new_object->skeleton.bone_count);
    	memcpy(new_object->skeleton.bones, obj->skeleton.bones, bonesArraySize);
    	memcpy(new_object->skeleton.pose,  obj->skeleton.pose,  bonesPoseSize);
    }

    if (obj->object.child_count)
    {
        if (obj->object.name[0]!='c')
        {
        	new_object->object.children = sCalloc(obj->object.child_count,sizeof(void*));
        }

        index_t d_children = 0;
        for (index_t i=0;i<object->child_count;i++)
        {
            old_child = object->children[i];
            if (old_child->inactive && old_child->name[0]=='o')
            {
            	old_child->inactive = 0;
            	continue;
            }
            new_child = new_object->object.children[d_children] = duplicate_object(obj->object.children[i]);
            if (new_child->name[0]!='b')
                new_child->parent = new_object;

            if (new_child->name[0]=='o' && new_object->object.name[0]=='o')
            {
                if (old_child->skeleton == obj->object.skeleton)
                    new_child->skeleton = new_object->object.skeleton;
            }
            d_children++;
        }
        new_object->object.child_count = d_children;
    }
    if (new_object->object.name[0]=='s')
    {
    	sSkeleton* skeleton = (sSkeleton*)new_object;
        for (index_t j=0;j<new_object->object.child_count;j++)
        {
            sObject* cb = new_object->object.children[j];
            if (cb->name[0]=='o' && (sObject*)cb->skeleton==object)
            {
                cb->skeleton=(sSkeleton*)new_object;
            }
            if (cb->name[0]=='b')
            {
                new_object->object.children[j] += (intptr)new_object->skeleton.bones - (intptr)((sSkeleton*)object)->bones;
            }
        }
        
        for (index_t i=0;i<skeleton->bone_count;i++)
        {
            sBone* sb = &skeleton->bones[i],*cb;
            sb->hash = scene->gobjects_unique_counter++;
            sb->skeleton = skeleton;
            sb->children = sCalloc(sb->child_count,sizeof(void*));
            memmove(sb->children,obj->skeleton.bones[i].children,sb->child_count*sizeof(void*));
            if (((sObject*)sb->parent)->name[0] == 'b')
            {
            	sb->parent += (intptr)new_object->skeleton.bones - (intptr)((sSkeleton*)object)->bones;
            }
            else
            {
            	sb->parent = skeleton;
            }
            for (index_t j=0;j<sb->child_count;j++)
            {
                cb = sb->children[j];
                if (cb->name[0]=='b')
                {
                    sb->children[j] += (intptr)new_object->skeleton.bones - (intptr)((sSkeleton*)object)->bones;
                }
            }
        }
    }
    if (obj->object.name[0]!='c')
    	sSceneAppendObject(new_object);
    return new_object;
}

void* sObjectDuplicate(void* obj)
{
    if (!obj) return 0;
    sObject* new = (sObject*)duplicate_object(obj);
    if (new->name[0]=='o')
    {
        sPhysicsAttach(new);
    }
    new->parent = 0;
    return new;
}

void* sObjectDuplicate2(void* obj)
{
    ((sObject*)obj)->parent = 0;
    return sObjectDuplicate(obj);
}

/*static void _remove_object_from_list(void* obj,void** list,index_t* list_length)
{
    index_t ind;
    for (ind=0;ind<*list_length;ind++)
    {
        if (obj==list[ind])
        {
            (*list_length)--;
            memmove(&list[ind],&list[ind+1],(*list_length-ind)*sizeof(void*));
            return;
        }
    }
}*/

static void _null_object_from_list(void* obj,void** list,index_t list_length)
{
    for (index_t ind=0;ind<list_length;ind++)
    {
        if (obj==list[ind])
        {
        	list[ind] = 0;
            return;
        }
    }
}

laType sObjectGlobalTransform(sObject* object)
{
    sObject* parent = object->parent;
    laType result = object->transform;
    laType pt = Identity;

    if (parent)
        pt = sObjectGlobalTransform(parent);

    else
        return result;

    return Mul(pt,result);
}

void _child_gt(sObject* object)
{
    for (index_t i=0;i<object->child_count;i++)
    {
    	sObject* child = (sObject*)object->children[i];
        child->transform_global = Mul(object->transform_global,child->transform);
        _child_gt(child);
    }
}

void sObjectPlaceChildren(sObject* object)
{
    //if (object->transformable)
    object->transform_global = object->transform;
    _child_gt(object);
}

static void _remove_children(void* obj)
{
    if (!obj) return;
	sObject* object = obj;
	sSkeleton* skel = obj;
	sScene* scene = object->scene;

    if (object->name[0]=='c') return;

    if (skel->name[0]=='s')
    {
		for (int b=0; b<skel->bone_count;b++)
		{
			sObject* bone = (sObject*)(skel->bones + b);
			sObjectRemoveParent(bone);
			sObjectSetParent(bone, skel,0);
		}
    }

	for (int i=0; i<object->child_count;i++)
	{
		sObject* child = object->children[i];
		if (child)
		{
			_remove_children(child);
			child->parent = 0;
			object->children[i] = 0;
		}
	}
	if (object->children)
	{
		sFree(object->children);
		object->children = 0;
		object->child_count = 0;
	}

	if (skel->name[0]=='s')
	{
    	for (uint32_t i=0;i<MAX_ACTION_LAYERS;i++)
    	{
    		if (skel->action[i].channels)
    			sFree(skel->action[i].channels);

    		if (skel->action[i].bones_matching)
    			sFree(skel->action[i].bones_matching);
    	}
    	sFree(skel->bones);
    	sFree(skel->pose);
	}

	_null_object_from_list(object, (void**)scene->gobjects,  scene->gobjects_count);
	_null_object_from_list(object, (void**)scene->objects,   scene->objects_count);
	_null_object_from_list(object, (void**)scene->lights,    scene->lights_count);
	_null_object_from_list(object, (void**)scene->skeletons, scene->skeletons_count);

    if (object->name[0]=='o')
    {
        sPhysicsRSFree(object);
        sPhysicsCSFree(object);
        if (object->geom) dGeomDestroy(object->geom);
        if (object->body) dBodyDestroy(object->body);
    }

    if (object->name[0]!='b' && object->name[0]!='c')
    {
        sFree(obj);
    }
}

void sObjectDelDuplicate(void* obj)
{
    sObject* object = obj;
    //sScene* scene = object->scene;
    if (!isDuplicate(obj) || !obj)
    {
        fprintf(stderr,"Object %s is not a duplicate\n",object->name);
        exit(-1);
    }
    else if (object->name[0]=='b')
    {
        fprintf(stderr,"Bone %s belongs to %s. It cannot be removed directly.\n",object->name,((sSkeleton*)((sBone*)object)->skeleton)->name);
        return;
    }
    else if (object->name[0]=='o')
    {
        sPhysicsCSFree(object);
        sPhysicsRSFree(object);
        sPhysicsRadSFree(object);
    }
    /*else if (object->name[0]=='s')
    {
    	sFree(((sSkeleton*)object)->bones);
    }*/

    sObjectDelParent(object);
    _remove_children(object);

    sSceneDefragLists(object->scene);
}

/*void sObjectEnd(sObject* obj)
{
	obj->data = 1;
}*/

void sObjectDelParent(void* obj)
{
    sObject* object = obj;
    if (object->parent==0) return;
    remove_child(object->parent,object,1);
    object->parent = 0;
}

void sObjectRemoveParent(void* obj)
{
    sObject* object = obj;
    if (!object->parent) return;
    sObjectDelParent(obj);
    object->transform = object->transform_global;

}

void sObjectSetParent(void* obj,void* parent,_Bool apply_transform)
{
    sObject* cobject = obj;
    sObject* pobject = parent;

    if (cobject == pobject)
    {
        printf("Trying to set parent to itself!!! (%s to %s)\n",cobject->name,pobject->name);
        return;
    }
    if (cobject->name[0]=='b' && pobject->name[0]=='b' && ((sBone*)pobject)->skeleton!=((sBone*)cobject)->skeleton)
    {
        fprintf(stderr,"Parent of bone should have same skeleton or other object (%s)->(%s)\n",cobject->name,pobject->name);
        exit(-1);
    }
    if (apply_transform)
    {
        laType parent_transform = Inverted(sObjectGlobalTransform(pobject));
        cobject->transform = Mul(parent_transform,cobject->transform);
    }
    add_child(parent,obj);
}

void sObjectSetSkeleton(sObject *obj,sSkeleton *skel)
{
	sObjectSetParent(obj,skel,0);
	obj->skeleton = skel;
}

uint32_t sObjectGetChildCount(sObject* object)
{
    return object->child_count;
}
sObject* sObjectGetChildren(sObject* object,uint32_t ind)
{
    return (ind<object->child_count) ? object->children[ind] : 0;
}
sObject* sObjectGetChild(sObject* object,char* name)
{
	for (int i=0;i<object->child_count;i++)
	{
		if (!strcmp(name, ((sObject*)object->children[i])->name))
		{
			return object->children[i];
		}
	}
	return 0;
}
sObject* sObjectGetParent(sObject* object)
{
    return object->parent;
}


_Bool isDuplicate(void* obj)
{
    sObject* child = obj;
    sScene* scene = child->scene;
    _Bool inactive=0;
    inactive |= ((intptr)child >= ((intptr)scene->skelets_inactive) && (intptr)child<((intptr)scene->skelets_inactive + scene->skelets_inactive_count*sizeof(sSkeleton)));
    inactive |= ((intptr)child >= ((intptr)scene->objects_inactive) && (intptr)child<((intptr)scene->objects_inactive + scene->objects_inactive_count*sizeof(sObject)));
    inactive |= ((intptr)child >= ((intptr)scene->lights_inactive) && (intptr)child<((intptr)scene->lights_inactive + scene->lights_inactive_count*sizeof(sLight)));
    return !inactive;
}

void sObjectSetTransformToPhysics(sObject* obj)
{
	if (obj->name[0] != 'o') return;
	laTypeD laRot = laTypeCastToDouble(&obj->transform_global);
	dReal *rot = laRot.a;
	if (obj->physicsType==2 || obj->physicsType==3)
	{
		rot[3] = rot[7] = rot[11] = 0.0;
		dBodySetPosition(obj->body,obj->transform_global.a[3],obj->transform_global.a[7],obj->transform_global.a[11]);
		dBodySetRotation(obj->body, rot);
	}
}

void sObjectPrintChildren(void* obj)
{
    sObject* object = obj;
    if (!object->child_count)
    {
        printf("%s hasn\'t children\n",object->name+1);
        return;
    }
    printf("Child list of %s (%d):\n",object->name+1,object->child_count);
    for (uint32_t i=0;i<object->child_count;i++)
    {
        sObject* child = object->children[i];
        printf("\t%s (%s)\n",child->name,isDuplicate(child) ? "duplicate" : "inactive");
    }
    printf("End of list. %d objects.\n",object->child_count);
}

void sObjectPrintHierarchy(void* obj, int recursion)
{
	sObject* object = obj;
	for (uint32_t i=0;i<object->child_count;i++)
	{
		sObject* child = object->children[i];
		for (int t=0;t<recursion;t++)
		{
			putchar(' ');putchar(' ');
		}
		printf("%s\n",child->name);
		sObjectPrintHierarchy(child, recursion+1);
	}
}

void sObjectTrackToOther(sObject* obj1,sObject* obj2,uint8_t look_axis,uint8_t up_axis)
{
    obj1->transform = LookAt(obj1->transform,obj2->transform_global,up_axis,look_axis);
    sObjectSetTransformToPhysics(obj1);
}

void sObjectTrackToPoint(sObject* obj1,laType point,uint8_t look_axis,uint8_t up_axis)
{
    obj1->transform = LookAt(obj1->transform,point,up_axis,look_axis);
    sObjectSetTransformToPhysics(obj1);
}

void sObjectIK(void* obj,void* elb,void* tar)
{
    sObject *object = obj;
    float l1 = 0.544298,l2 = 0.544298;
    if (!object->parent) return;
    
    sObject *elbow = elb;
    sObject *target = tar;
    sObject *par = object->parent;
    laType par_pos,elb_pos,tar_pos;
    laType halfph,par_tar,hpt_elb;
    laType axis;float angle;
    
    if (!obj || !tar)
    {
        return;
    }
    if (!par) goto object_track;
    if (par->name[0]=='b') l1=((sBone*)par)->length;
    if (object->name[0]=='b') l2=((sBone*)object)->length;
    par_pos = GetPosition(par->transform_global);
    elb_pos = GetPosition(elbow->transform_global);
    tar_pos = GetPosition(target->transform_global);
    
    halfph = Vector((tar_pos.a[0] + par_pos.a[0])/2,
                    (tar_pos.a[1] + par_pos.a[1])/2,
                    (tar_pos.a[2] + par_pos.a[2])/2);
    par_tar = Vector((tar_pos.a[0]- par_pos.a[0]),
                     (tar_pos.a[1] - par_pos.a[1]),
                     (tar_pos.a[2] - par_pos.a[2]));
    
    hpt_elb = Vector(elb_pos.a[0] - halfph.a[0],
                     elb_pos.a[1] - halfph.a[1],
                     elb_pos.a[2] - halfph.a[2]);
    
    float d = Length(par_tar);
    
    /*laType obj_tar = Vector((tar_pos.a[0]- obj_pos.a[0]),
     *                                                        (tar_pos.a[1] - obj_pos.a[1]),
     *                                                        (tar_pos.a[2] - obj_pos.a[2]));*/
    par->transformable = 0;
    if (d>l1+l2)
    {
        par->transform_global = LookAt(par->transform_global,target->transform_global,0,2);
        //object->transform_global = LookAt(object->transform_global,target->transform_global,0,2);
        sObjectPlaceChildren(par);
    }
    else
    {
        axis = Crossn(par_tar,hpt_elb);
        angle = acosf(d/(l1+l2));
        par->transform_global = LookAt(par->transform_global,target->transform_global,0,2);
        RotateByAxis(&par->transform_global,axis,angle);
        sObjectPlaceChildren(par);
        RotateByAxis(&object->transform_global,axis,-2*angle);
    }
    par->transformable = 1;
    return;
    object_track:
    object->transform_global = LookAt(object->transform_global,target->transform_global,0,2);
    sObjectSetTransformToPhysics(obj);
}

laType sObjectGetPositionGlobal3fv(sObject *object)
{
	return Vector(object->transform_global.a[3], object->transform_global.a[7],object->transform_global.a[11]);
}

void sObjectSetPositionGlobal3fv(void *obj, laType vector)
{
	sObject *object = obj;
	if (vector.type == VECTOR)
	{
		object->transform_global.a[3] = vector.a[0];
		object->transform_global.a[7] = vector.a[1];
		object->transform_global.a[11]= vector.a[2];
	}
	else if (vector.type == MATRIX)
	{
		object->transform_global.a[3] = vector.a[3];
		object->transform_global.a[7] = vector.a[7];
		object->transform_global.a[11]= vector.a[11];
	}
	if (object->parent)
	{
		object->transform = Mul(Inverted(((sObject*)object->parent)->transform_global), object->transform_global);
	}
	else
	{
		object->transform = object->transform_global;
	}
	sObjectSetTransformToPhysics(object);
}

void sObjectSetLocalTransform(void* obj,laType transform)
{
	if (transform.type != MATRIX) return;
	sObject *object = obj;
	object->transform_global = object->transform = transform;
	sObjectSetTransformToPhysics(object);
}

void sObjectMoveGlobal3fv(void* object,laType vector)
{
    sObject* obj = object;
    if (vector.type == MATRIX)
    {
        vector.type = VECTOR;
        vector.a[0] = vector.a[3];
        vector.a[1] = vector.a[7];
        vector.a[2] = vector.a[11];
    }
    
    if (obj->parent)
        vector = Mul(InvertedFast(((sObject*)obj->parent)->transform_global),vector);
    
    if (obj->name[0]=='o' && obj->physics_enabled)
    {
        if (obj->physicsType==1)
        {
            obj->transform = Add(obj->transform,Mul(vector,GetOrientation(obj->transform)));
            if (obj->parent)
                obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
            else
                obj->transform_global = obj->transform;
            dGeomSetPosition(obj->geom,obj->transform_global.a[3],obj->transform_global.a[7],obj->transform_global.a[11]);
        }
        else if (obj->physicsType==2 || obj->physicsType==3)
        {
            obj->transform = Add(obj->transform,Mul(vector,GetOrientation(obj->transform)));
            if (obj->parent)
                obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
            else
                obj->transform_global = obj->transform;
            dBodySetPosition(obj->body,obj->transform_global.a[3],obj->transform_global.a[7],obj->transform_global.a[11]);
        }
        else
        {
            obj->transform = Add(obj->transform,vector);
        }
    }
    else
    {
        obj->transform = Add(obj->transform,vector);
    }
}

void sObjectMoveLocal3fv(void* object,laType vector)
{
    sObject* obj = object;
    if (vector.type == MATRIX)
    {
        vector.type = VECTOR;
        vector.a[0] = vector.a[3];
        vector.a[1] = vector.a[7];
        vector.a[2] = vector.a[11];
    }
    
    if (obj->parent)
        vector = Mul(obj->transform,vector);
    
    if (obj->name[0]=='o' && obj->physics_enabled)
    {
        if (obj->physicsType==1)
        {
            obj->transform = Add(obj->transform,Mul(vector,GetOrientation(obj->transform)));
            if (obj->parent)
                obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
            else
                obj->transform_global = obj->transform;
            dGeomSetPosition(obj->geom,obj->transform_global.a[3],obj->transform_global.a[7],obj->transform_global.a[11]);
        }
        else if (obj->physicsType==2 || obj->physicsType==3)
        {
            obj->transform = Add(obj->transform,Mul(vector,GetOrientation(obj->transform)));
            if (obj->parent)
                obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
            else
                obj->transform_global = obj->transform;
            dBodySetPosition(obj->body,obj->transform_global.a[3],obj->transform_global.a[7],obj->transform_global.a[11]);
        }
        else
        {
            obj->transform = Add(obj->transform,vector);
        }
    }
    else
    {
        obj->transform = Add(obj->transform,vector);
    }
}

void sObjectSetRotation3f(void* object,float x,float y,float z)
{
    sObject* obj = object;
    dReal* oldrot;
    laType rotation = RotationXYZ(x,y,z);
    
    if (obj->name[0]=='o' && (obj->physicsType==2 || obj->physicsType==3) && obj->physics_enabled)
    {
        oldrot = (dReal*)dBodyGetRotation(obj->body);
        for (uint8_t i=0;i<12;i++)
        {
            if (i!=3 && i!=7 && i!=11 && i!=15)
                oldrot[i] = rotation.a[i];
        }
        dBodySetRotation(obj->body,oldrot);
    }
    else
    {
        for (uint8_t i=0;i<12;i++)
        {
            if (i!=3 && i!=7 && i!=11 && i!=15)
                obj->transform.a[i] = rotation.a[i];
        }
		if (obj->parent)
			obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
		else
			obj->transform_global = obj->transform;
    }
}

void sObjectRotateLocal3f(void* object,float x,float y,float z)
{
    sObject* obj = object;
    
    laType newrot;
    laTypeD newrotd;
    dReal* oldrot;
    
    if (obj->name[0]=='o' && (obj->physicsType==2 || obj->physicsType==3) && obj->physics_enabled)
    {
        oldrot = (dReal*)dBodyGetRotation(obj->body);
        newrot = Matrix3x3(oldrot[0],oldrot[1],oldrot[2],
                           oldrot[4],oldrot[5],oldrot[6],
                           oldrot[8],oldrot[9],oldrot[10]);
        RotateXYZlocal(&newrot,x,y,z);
        newrotd = laTypeCastToDouble(&newrot);
        dBodySetRotation(obj->body,newrotd.a);
    }
    else
    {
        RotateXYZlocal(&obj->transform,x,y,z);
        if (obj->parent)
            obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
        else
            obj->transform_global = obj->transform;
    }
}

void sObjectRotateGlobal3f(void* object,float x,float y,float z)
{
    sObject* obj = object;

    laType newrot;
    laTypeD newrotd;
    dReal* oldrot;

    if (obj->name[0]=='o' && (obj->physicsType==2 || obj->physicsType==3) && obj->physics_enabled)
    {
        oldrot = (dReal*)dBodyGetRotation(obj->body);
        newrot = Matrix3x3(oldrot[0],oldrot[1],oldrot[2],
                           oldrot[4],oldrot[5],oldrot[6],
                           oldrot[8],oldrot[9],oldrot[10]);
        RotateXYZglobal(&newrot,x,y,z);
        newrotd = laTypeCastToDouble(&newrot);
        dBodySetRotation(obj->body,newrotd.a);
    }
    else
    {
        RotateXYZglobal(&obj->transform,x,y,z);
        if (obj->parent)
            obj->transform_global = Mul(((sObject*)obj->parent)->transform_global,obj->transform);
        else
            obj->transform_global = obj->transform;
    }
}

laType sObjectGetVectorTo(void* object,void* object_to)
{
    return Sub(GetPosition(((sObject*)object_to)->transform_global),GetPosition(((sObject*)object)->transform_global));
}
float sObjectGetDistanceTo(void* object,void* object_to)
{
    return Length(Sub(GetPosition(((sObject*)object_to)->transform_global),GetPosition(((sObject*)object)->transform_global)));
}

void sLightSetColor(void* light, float color[4])
{
    if (!light) return;
    sLight* li = light;
    if (li->name[0]!='l')
    {
        fprintf(stderr,"Warning sLightSetColor: %s is not a light\n",li->name);
        return;
    }
    memcpy(&li->color, color, sizeof(float[4]));
}

void sObjectSetBehaviour(sObject* ob, fptr func)
{
    ob->behaviour = func;
}
