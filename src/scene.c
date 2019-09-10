/*
 * scene.c
 *
 *  Created on: 25 дек. 2017 г.
 *      Author: ivan
 */

#include "engine.h"
#define get_index_by_name(name,objects,length) _get_index_by_name(objects,sizeof(objects[0]),name,length)
#define get_index_by_hash(hash,objects,length) _get_index_by_hash(objects,sizeof(objects[0]),hash,length)

extern sShader base_shader,skin_shader,shader_shadowmap,skin_shader_shadowmap;
extern sShader vectorsShader,skin_vectorsShader;
extern sShader voxelShader,skin_voxelShader;
extern sScene* active_scene;

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
static size_s _get_index_by_hash(void* objects,size_s size,uint32_t hash,size_s length)
{
	uint32_t index=0;
	for (;index<length;index++)
	{
		if (hash==*(uint32_t*)(((char*)objects)+index*size))
			return index;
	}
	return -1;
}

void readf(void* data_ptr,uint32_t size, uint32_t count, FILE* file_ptr)
{
	if (fread(data_ptr,size,count,file_ptr)<count)
	{
		printf("Error: Unexpected end of file\n");
		exit(-1);
	}
}

void readstr(char* to,int count,FILE* fp)
{
	for (uint32_t i=0;i<count;i++)
	{
		readf(to+i,1,1,fp);
		if (to[i]=='\n')
		{
			to[i]=0;
			return;
		}
	}
	return;
	if (!fgets(to,count,fp))
	{
		fprintf(LOGOUT, "Unexpected end of file\n");
		exit(-1);
	}
	while (*to++ != '\n' && count-->0){};
	*(to-1) = 0;
}

static void load_camera(sScene* scene,FILE* fp)
{
	sCamera* camera = &scene->camera;
	char parent_name[255];
    camera->scene = scene;
	camera->children = 0;
	camera->parent = 0;
	camera->child_count = 0;

	readstr(camera->name, 255, fp);
	readstr(parent_name,  255, fp);
	printf("%s parent is %s\n", camera->name, parent_name);
	camera->hash = S_Name2hash(camera->name);
	camera->parent = (void*)(intptr)S_Name2hash(parent_name);
// Поворот
	camera->transform.type = MATRIX;
	readf(camera->transform.a,1,sizeof(camera->transform.a),fp);
	//camera->transform = Identity;
	camera->transform_global_previous = camera->transform_global = camera->transform;
	camera->zNear = 0.02;
	camera->zFar = 400.0;
	//printf("%s\n",camera->name);
	//LAPrint(camera->transform);

	/*float vec[3] = {camera->transform.a[3],camera->transform.a[7],camera->transform.a[11]};
	camera->transform.a[3] = 0;
	camera->transform.a[7] = 0;
	camera->transform.a[11]= 0;

	camera->transform = Mul(RotationY(3.1415926535/2),camera->transform);
	camera->scene = scene;
	Translate(&camera->transform,-vec[0],-vec[1],-vec[2]);

	//camera->transform = Inverted(camera->transform);
	process_camera(camera);
	RotateXYZlocal(&camera->orientation,-3.1415926535/2,0.0,0.0);*/
	readf(&camera->FOV,1,4,fp);
	camera->view_point = 0;
	camera->projection = Perspective(camera->width, camera->height, camera->zFar, camera->zNear, camera->FOV);
	camera->viewProjection = Mul(camera->projection,Inverted(camera->transform_global));
	camera->transformable = 1;
}

static void load_light(sScene* scene,FILE* fp)
{
	char parent_name[256];
	sLight* light = &(scene->lights_inactive[scene->lights_inactive_count++]);
	light->scene = scene;
	readstr(light->name,255,fp);
	//printf("%s\n",light->name);

	light->hash = S_Name2hash(light->name);
	light->transform.type = MATRIX;
	readf(&light->inactive,1,1,fp);

	readstr(parent_name, 255, fp);
	light->parent = (void*)(intptr)S_Name2hash(parent_name);

	readf(light->transform.a,1,sizeof(light->transform.a),fp);
	readf(&light->color,1,sizeof(float)*4,fp);
	readf(&light->type,1,1,fp);
	readf(&light->shadow,1,1,fp);
	readf(&light->inner,1,4,fp);
	readf(&light->outer,1,4,fp);

	light->FOV = light->outer;
	light->inner = cos(light->inner/360.0*3.1415926353);
	light->outer = cos(light->outer/360.0*3.1415926353);

	readf(&light->zNear,1,4,fp);// = 0.1;
	readf(&light->zFar, 1,4,fp);// = 50.0;

	if (light->type == S_SUN)
	{
		light->height = light->width = S_SUN_SHADOW_SIZE;
		light->projection = Ortho(40.0, light->zFar, light->zNear);
	}
	else
	{
		light->height = light->width = S_SPOT_SHADOW_SIZE;
		light->projection = Perspective(S_SPOT_SHADOW_SIZE, S_SPOT_SHADOW_SIZE, light->zFar, light->zNear, light->FOV);
	}

	//fprintf(LOGOUT, "zFar %f, zNear %f\n",light->zFar,light->zNear);

	if (light->shadow)
	{
		//printf("sCameraInitShadowFB(scene->lights[\"%s\"])\n",light->name);
		sCameraInitShadowFB((sCamera*)light);
	}

	//SetCameraDirection(&light->transform,Vector(light->transform.a[1],light->transform.a[5],light->transform.a[9]));
	light->transform_global_previous = light->transform_global = light->transform;
	light->viewProjection = Mul(light->projection,Inverted(light->transform_global));
}

static void load_mesh(sScene* scene,FILE* sp)
{
	sMesh* mesh = &(scene->meshes[scene->mesh_count++]);
	size_s index = 0;
	char name[256];
	mesh->link_matrix = 0;

	readstr(name,255,sp);
	readstr(buff,255,sp);

	//fprintf(LOGOUT, "%s\n", name);

	mesh->hash = S_Name2hash(name);
	index = get_index_by_name(buff,scene->materials,scene->material_count);

	mesh->material = (int)index==-1 ? scene->materials : &(scene->materials[index]);
	sMeshLoad(mesh, 0, name);
}

static void load_texture(sScene* scene,FILE* fp)
{
	/*IMAGE tex;
	float aniso;*/
	sTexture* texture = &(scene->textures[scene->texture_count]);
	readstr(texture->name,255,fp);

	texture->hash = S_Name2hash(texture->name);
	sprintf(buff,RESDIR"/");
	readstr(&buff[strlen(RESDIR"/")],250,fp);
	strcat(buff,".dds");

	glc(sTextureLoadDDS(texture,buff));
	scene->texture_count++;
}

static void load_material(sScene* scene,FILE* fp)
{
	sMaterial* material = &(scene->materials[scene->material_count++]);
	memset(material, 0, sizeof(sMaterial));

	char name[4][256];

	readstr(material->name,255,fp);

	//fprintf(LOGOUT, "%s\n",material->name);

	readstr(name[0],255,fp);
	readstr(name[1],255,fp);
	readstr(name[2],255,fp);
	readstr(name[3],255,fp);
	readf(&material->diffuse,1,12,fp);
	readf(&material->specular,1,12,fp);
	readf(&material->glow,1,4,fp);
	readf(&material->transparency,1,4,fp);
	readf(&material->friction,1,8,fp);

	material->height_scale = 1.0;

	//printf("friction is %f\n",material->friction);

	material->hash = S_Name2hash(material->name);

	size_s index = get_index_by_name(name[0],scene->textures,scene->texture_count);
	material->diffuse_texture = (int)index==-1 ? 0 : &(scene->textures[index]);
	index = get_index_by_name(name[1],scene->textures,scene->texture_count);
	material->specular_texture = (int)index==-1 ? 0 : &(scene->textures[index]);
	index = get_index_by_name(name[2],scene->textures,scene->texture_count);
	material->height_texture = (int)index==-1 ? 0 : &(scene->textures[index]);
	index = get_index_by_name(name[3],scene->textures,scene->texture_count);
	material->lightmap_texture = (int)index==-1 ? 0 : &(scene->textures[index]);

	//readf(&material->shadeless,1,1,fp);
}

static void load_object(sScene* scene,FILE* fp)
{
	char mesh_name[256];
	float mass;
	uint32_t index;
	sObject* object = &(scene->objects_inactive[scene->objects_inactive_count++]);
	object->scene = scene;
	readstr(object->name,255,fp);
	readf(&object->inactive,1,1,fp);

	readstr(mesh_name,255,fp);

	object->parent = (void*)(intptr)S_Name2hash(mesh_name);


	object->hash = S_Name2hash(object->name);
	readstr(mesh_name,255,fp);

	readf(object->transform.a,1,sizeof(object->transform.a),fp);
	object->transform.type = MATRIX;

	object->transform_global_previous = object->transform_global = object->transform;
	readf(&object->physicsType,1,4,fp);
	readf(&object->physicsShape,1,4,fp);
	readf(&mass,1,4,fp);
	object->physicsMass = mass;
	object->collisionGroups = 1;
	object->collideWithGroups = 1;
	index = get_index_by_name(mesh_name,scene->meshes,scene->mesh_count);
	object->mesh = index==-1 ? 0 : &(scene->meshes[index]);

	object->hidden = 0;
	object->ghost = 0;
	object->physics_enabled = 1;

	memset(&object->ray,0,sizeof(object->ray));
	memset(&object->collider,0,sizeof(object->collider));

	if (object->mesh->deformed)
	{
		//fprintf(LOGOUT, "%s have deformable mesh\n", object->name);
		readstr(mesh_name,255,fp);
		index = get_index_by_name(mesh_name,scene->skelets_inactive,scene->skelets_inactive_count);
		object->skeleton = &scene->skelets_inactive[index];

		for (index_t i=0;i<object->skeleton->bone_count;i++)
		{
			object->mesh->link_matrix[i] = Mul(InvertedFast(object->transform),object->mesh->link_matrix[i]);
		}
		//sObjectSetParent(object,object->skeleton,0);
	}
	else
	{
		object->skeleton = 0;
	}
}

void load_bone(sSkeleton* skelet,FILE* fp)
{
	uint32_t parent_index;
	sBone* bone = &skelet->bones[skelet->bone_count++];
	sScene* scene = skelet->scene;
	bone->transform.type = MATRIX;
	bone->transformable = 1;
	bone->animated = 1;
	bone->skeleton = skelet;
	readstr(bone->name,63,fp);
	readf(&parent_index,1,4,fp);
	readf(&bone->length,1,4,fp);
	readf(bone->transform.a,1,sizeof(bone->transform.a),fp);
	// Убрать!!!
	//bone->transform = Identity;
	bone->transform_global_previous = bone->transform_global = bone->transform;

	bone->parent = (void*)(intptr)parent_index;
	bone->hash = S_Name2hash(bone->name);

	uint32_t index = get_index_by_name("mbone",scene->meshes,scene->mesh_count);
	if (index!=0xFFFFFFFF)
	{
		bone->mesh = &scene->meshes[index];
	}
	else
	{
		bone->mesh = 0;
	}
	bone->scene = scene;
}

void load_skeleton(sScene* scene,FILE* fp)
{
	sSkeleton* object = &(scene->skelets_inactive[scene->skelets_inactive_count++]);
	sBone* bone;
	uint32_t bone_count;
	char parent_name[256];

	object->transform.type = MATRIX;

	object->scene = scene;
	readstr(object->name,255,fp);
	readf(&object->inactive,1,1,fp);
	readstr(parent_name,255,fp);
	object->parent = (void*)(intptr_t)S_Name2hash(parent_name);
	//fprintf(LOGOUT,"%s\n", object->name);

	readf(object->transform.a,1,sizeof(object->transform.a),fp);
	readf(&bone_count,1,4,fp);
	object->transform_global = object->transform;

	object->hash  = S_Name2hash(object->name);
	object->bones = sCalloc(sizeof(sBone),  bone_count);
	object->pose  = sCalloc(sizeof(laType), bone_count);

	for (object->bone_count=0;object->bone_count<bone_count;load_bone(object,fp));
	for (uint32_t b=0;b<object->bone_count;b++)
	{
		object->pose[b] = object->bones[b].transform;
		if ((intptr)object->bones[b].parent == 0xFFFFFFFF)
		{
			sObjectSetParent(&object->bones[b],object,0);
		}
		else
		{
			bone = &object->bones[(intptr)object->bones[b].parent];
			object->bones[b].parent = 0;
			sObjectSetParent(&object->bones[b],bone,0);
		}
	}
}

static void load_action(sScene* scene,FILE* fp)
{
	sAction* action = &scene->actions[scene->actions_count++];
	readstr(action->name,256,fp);
	sprintf(buff,"%s/mesh/%s%s",RESDIR, action->name, ACTIONRES);
	FILE* ap = fopen(buff,"rb");
	if (!ap)
	{
		fprintf(stderr,"Action file %s not found\n",buff);
		exit(-1);
	}
	uint32_t bones=0;
	uint32_t frames=0;
	
    // skip checksum
    readf(&bones,1,4,ap);
	readf(&bones,1,4,ap);
    //
    
	readf(&bones,1,4,ap);
	readf(&frames,1,4,ap);
	action->channels_count = bones;
	action->channels = sCalloc(sizeof(sActionChannel),action->channels_count);
	//fprintf(LOGOUT, "Bones %d, frames %d\n",bones,frames);
	for (index_t channel=0;channel<bones;channel++)
	{
		readstr(action->channels[channel].name,256,ap);
		//printf("load action %s for bone %s\n",action->name,action->channels[channel].name);
		action->channels[channel].keyframes = sCalloc(sizeof(laType),frames);
		for (index_t f=0;f<frames;f++)
		{
			readf(action->channels[channel].keyframes[f].a,4,16,ap);
			action->channels[channel].keyframes[f].type = MATRIX;
		}
		action->channels[channel].keyframes_count = frames;
		//LAPrint(action->channels[channel].keyframe[1]);
	}
}

void sActionLoad(sScene* scene,char* path,char* name)
{

	for (size_s i=0;i<scene->actions_count;i++)
	{
		if (!strcmp(name,scene->actions[i].name)) return;
	}
	scene->actions = sRealloc(scene->actions,(scene->actions_count+1)*sizeof(sAction));
	sAction* action = &scene->actions[scene->actions_count++];

	FILE* ap = fopen(path,"rb");
	if (!ap)
	{
		fprintf(stderr,"Action file %s not found\n",path);
		exit(-1);
	}
	uint32_t bones=0;
	uint32_t frames=0;

	strcpy(action->name,name);

	readf(&bones,1,4,ap);
	readf(&frames,1,4,ap);
	action->channels_count = bones;
	action->channels = sCalloc(sizeof(sActionChannel),action->channels_count);
	//fprintf(LOGOUT, "Bones %d, frames %d\n",bones,frames);
	for (index_t channel=0;channel<bones;channel++)
	{
		readstr(buff,255,ap);
		//printf("load action %s for bone %s\n",action->name,buff);
		action->channels[channel].keyframes = sCalloc(sizeof(laType),frames);
		for (index_t f=0;f<frames;f++)
		{
			readf(action->channels[channel].keyframes[f].a,4,16,ap);
			action->channels[channel].keyframes[f].type = MATRIX;
		}
		action->channels[channel].keyframes_count = frames;
		//LAPrint(action->channels[channel].keyframe[1]);
	}
}

sMesh* sSceneAddMesh(sScene* scene, char* name)
{
	for (uint32_t i=0; i<scene->mesh_count; i++)
	{
		if (scene->meshes[i].hash)
		{
			sMeshLoad(scene->meshes + i, scene->materials, name);
			return scene->meshes + i;
		}
	}
	scene->meshes = sRealloc(scene->meshes, sizeof(sMesh[scene->mesh_count + 1]));
	sMeshLoad(scene->meshes + scene->mesh_count, scene->materials, name);
	scene->mesh_count++;
	return scene->meshes + scene->mesh_count - 1;
}

void sSceneRemoveMesh(sScene* scene, char* name)
{
	uint32_t hash = S_Name2hash(name);
	for (uint32_t i=0; i<scene->mesh_count; i++)
	{
		if (scene->meshes[i].hash == hash)
		{
			sMeshDelete(scene->meshes + i);
			memset(scene->meshes, 0, sizeof(sMesh));
		}
	}
}

void S_SCENE_Close(sScene* scene)
{
	for (uint32_t i=0;i<scene->mesh_count;i++)
	{
		sMeshDeleteBuffers(scene->meshes + i);
	}
	for (uint32_t i=0;i<scene->texture_count;i++)
	{
		sTextureFree(scene->textures + i);
	}

	sFree(scene->textures);
	sFree(scene->materials);
	sFree(scene->meshes);
	sFree(scene->lights_inactive);
	sFree(scene->objects_inactive);
}

void* sSceneGetObject(sScene* scene,char* name)
{
	//uint32_t hash = S_Name2hash(name);
	for (index_t i=0;i<scene->gobjects_count;i++)
	{
		if (!strcmp(name, ((sObject*)scene->gobjects[i])->name))
		{
			return scene->gobjects[i];
		}
	}
	return 0;
}

void sSceneLoad(sScene* scene,char* name)
{
	memset(scene,0,sizeof(sScene));

	sprintf(buff,"%s/scenes/%s.scene",RESDIR,name);

	FILE* fp = fopen(buff,"rb");
	if (!fp)
	{
		fprintf(LOGOUT, "not such file %s\n",buff);
		exit(-1);
	}
	//printf("Loading camera\n");
	scene->camera.view_point = 0;
	load_camera(scene,fp);

	uint32_t mesh_count = 0;
	uint32_t texture_count = 0;
	uint32_t material_count = 0;
	uint32_t light_count = 0;
	uint32_t object_count = 0;
	uint32_t skelet_count = 0;
	uint32_t action_count = 0;

	readf(&texture_count,1,4,fp);
	fprintf(LOGOUT, "Load %d textures\n",texture_count);
	scene->textures = sCalloc(texture_count+5,sizeof(sTexture));
	for (scene->texture_count=0;scene->texture_count<texture_count;load_texture(scene,fp));

	fprintf(LOGOUT, "Load materials\n");
	readf(&material_count,1,4,fp);
	scene->materials = sCalloc(material_count,sizeof(sMaterial));
	for (scene->material_count=0;scene->material_count<material_count;load_material(scene,fp));

	fprintf(LOGOUT, "Load meshes\n");
	readf(&mesh_count,1,4,fp);
	scene->meshes = sCalloc(mesh_count,sizeof(sMesh));
	for (scene->mesh_count=0;scene->mesh_count<mesh_count;load_mesh(scene,fp));

	fprintf(LOGOUT, "Load actions\n");
	readf(&action_count,1,4,fp);
	scene->actions = sCalloc(action_count,sizeof(sAction));
	for (scene->actions_count=0;scene->actions_count<action_count;load_action(scene,fp));

	fprintf(LOGOUT, "Load lights\n");
	readf(&light_count,1,4,fp);
	scene->lights_inactive = sCalloc(light_count,sizeof(sLight));
	for (scene->lights_inactive_count=0;scene->lights_inactive_count<light_count;load_light(scene,fp));
	glc();

	fprintf(LOGOUT, "Load skeleton\n");
	readf(&skelet_count,1,4,fp);
	scene->skelets_inactive = sCalloc(skelet_count,sizeof(sSkeleton));
	for (scene->skelets_inactive_count=0;scene->skelets_inactive_count<skelet_count;load_skeleton(scene,fp));

	fprintf(LOGOUT, "Load objects\n");
	readf(&object_count,1,4,fp);
	scene->objects_inactive = sCalloc(object_count,sizeof(sObject));
	for (scene->objects_inactive_count=0;scene->objects_inactive_count<object_count;load_object(scene,fp));

	scene->gobjects_count = 1;
	scene->gobjects = sCalloc(1,sizeof(void*));
	scene->gobjects[0] = (void*)&scene->camera;

	fprintf(LOGOUT, "Inverting link matrices\n");
	for (index_t i=0;i<scene->mesh_count;i++)
	{
		if (scene->meshes[i].deformed)
		{
			void* kk = (uint64_t*)scene->meshes[i].link_matrix;
			uint32_t bone_count = *(size_s*)(kk-8);
			bone_count /= sizeof(laType);

			for (index_t b=0;b<bone_count;b++)
			{
				scene->meshes[i].link_matrix[b] = Inverted(scene->meshes[i].link_matrix[b]);
			}
		}
	}

	if (scene->camera.parent)
	{
		uint32_t parent_index = get_index_by_hash((intptr)scene->camera.parent,scene->objects_inactive,scene->objects_inactive_count);
		if (parent_index!=0xFFFFFFFF)
		{
			sObjectSetParent(&scene->camera,&scene->objects_inactive[parent_index],0);
		}
		else
		{
			parent_index = get_index_by_hash((intptr)scene->camera.parent,scene->lights_inactive,scene->lights_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(&scene->camera,&scene->lights_inactive[parent_index],0);
			}
			else
			{
				parent_index = get_index_by_hash((intptr)scene->camera.parent,scene->skelets_inactive,scene->skelets_inactive_count);
				if (parent_index!=0xFFFFFFFF)
				{
					sObjectSetParent(&scene->camera,&scene->skelets_inactive[parent_index],0);
				}
			}
		}
	}

	for (index_t i=0;i<scene->objects_inactive_count;i++)
	{
		if (scene->objects_inactive[i].parent)
		{
			sObject* obj = scene->objects_inactive + i;
			//printf("objects_inactive %d\n",scene->objects_inactive_count);
			if ((uintptr_t)obj->parent==scene->camera.hash)
			{
				sObjectSetParent(obj,&scene->camera,0);
				continue;
			}
			uint32_t
			parent_index = get_index_by_hash((intptr)obj->parent,scene->objects_inactive,scene->objects_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->objects_inactive[parent_index],0);
				continue;
			}
			//printf("lights_inactive %d\n",scene->lights_inactive_count);
			parent_index = get_index_by_hash((intptr)obj->parent,scene->lights_inactive,scene->lights_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->lights_inactive[parent_index],0);
				continue;
			}
			//printf("skelets_inactive %d\n",scene->skelets_inactive_count);
			parent_index = get_index_by_hash((intptr)obj->parent,scene->skelets_inactive,scene->skelets_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->skelets_inactive[parent_index],0);
				continue;
			}
			//fprintf(stderr,"Something wrong in loaded scene\n");
			//exit(-1);
		}
	}

	for (index_t i=0;i<scene->lights_inactive_count;i++)
	{
		if (scene->lights_inactive[i].parent)
		{
			sObject* obj = (sObject*)scene->lights_inactive + i;
			//printf("objects_inactive %d\n",scene->objects_inactive_count);
			if ((uintptr_t)obj->parent==scene->camera.hash)
			{
				sObjectSetParent(obj,&scene->camera,0);
				continue;
			}
			uint32_t
			parent_index = get_index_by_hash((intptr)obj->parent,scene->objects_inactive,scene->objects_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->objects_inactive[parent_index],0);
				continue;
			}
			//printf("lights_inactive %d\n",scene->lights_inactive_count);
			parent_index = get_index_by_hash((intptr)obj->parent,scene->lights_inactive,scene->lights_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->lights_inactive[parent_index],0);
				continue;
			}
			//printf("skelets_inactive %d\n",scene->skelets_inactive_count);
			parent_index = get_index_by_hash((intptr)obj->parent,scene->skelets_inactive,scene->skelets_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->skelets_inactive[parent_index],0);
				continue;
			}
			//fprintf(stderr,"Something wrong in loaded scene\n");
			//exit(-1);
		}
	}

	for (index_t i=0;i<scene->skelets_inactive_count;i++)
	{
		if (scene->skelets_inactive[i].parent)
		{
			sObject* obj = (sObject*)scene->skelets_inactive + i;
			//printf("objects_inactive %d\n",scene->objects_inactive_count);
			if ((uintptr_t)obj->parent==scene->camera.hash)
			{
				sObjectSetParent(obj,&scene->camera,0);
				continue;
			}
			uint32_t
			parent_index = get_index_by_hash((intptr)obj->parent,scene->objects_inactive,scene->objects_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->objects_inactive[parent_index],0);
				continue;
			}
			//printf("lights_inactive %d\n",scene->lights_inactive_count);
			parent_index = get_index_by_hash((intptr)obj->parent,scene->lights_inactive,scene->lights_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->lights_inactive[parent_index],0);
				continue;
			}
			//printf("skelets_inactive %d\n",scene->skelets_inactive_count);
			parent_index = get_index_by_hash((intptr)obj->parent,scene->skelets_inactive,scene->skelets_inactive_count);
			if (parent_index!=0xFFFFFFFF)
			{
				sObjectSetParent(obj,&scene->skelets_inactive[parent_index],0);
				continue;
			}
			//fprintf(stderr,"Something wrong in loaded scene\n");
			//exit(-1);
		}
	}

	fprintf(LOGOUT, "Initializing Open Dynamics Engine\n");
	sPhysicsInit(scene);
	//fprintf(LOGOUT, "Adding objects\n");
	for (index_t i=0;i<scene->lights_inactive_count;i++)
	{
		if (!scene->lights_inactive[i].parent && !scene->lights_inactive[i].inactive)
		{
			//printf("Adding %s\n",scene->lights_inactive[i].name);
			sObjectDuplicate(&scene->lights_inactive[i]);
		}
	}
	for (index_t i=0;i<scene->objects_inactive_count;i++)
	{
		if (!scene->objects_inactive[i].parent && !scene->objects_inactive[i].inactive)
		{
			//printf("Adding %s\n",scene->objects_inactive[i].name);
			sObjectDuplicate(&scene->objects_inactive[i]);
		}
	}
	for (index_t i=0;i<scene->skelets_inactive_count;i++)
	{
		if (!scene->skelets_inactive[i].parent && !scene->skelets_inactive[i].inactive)
		{
			//printf("Adding %s\n",scene->skelets_inactive[i].name);
			sObjectDuplicate(&scene->skelets_inactive[i]);
		}
	}
	for (index_t i=0;i<scene->objects_inactive_count;i++)
		scene->objects_inactive[i].inactive = 0;
	for (index_t i=0;i<scene->lights_inactive_count;i++)
		scene->lights_inactive[i].inactive = 0;
	for (index_t i=0;i<scene->skelets_inactive_count;i++)
		scene->skelets_inactive[i].inactive = 0;
	fclose(fp);

    scene->shader_list[0] = &base_shader;
    scene->shader_list[1] = &skin_shader;
    scene->shader_list[2] = &shader_shadowmap;
    scene->shader_list[3] = &skin_shader_shadowmap;
    scene->shader_list[4] = &vectorsShader;
    scene->shader_list[5] = &skin_vectorsShader;
    scene->shader_list[6] = &voxelShader;
    scene->shader_list[7] = &skin_voxelShader;

    sCameraInitFB(&scene->camera);

	for (index_t i=0;i<scene->gobjects_count;i++)
	{
		if (((sObject*)scene->gobjects[i])->parent) continue;
		sObjectPlaceChildren((sObject*)scene->gobjects[i]);
	}
}

sMaterial* sSceneGetMaterial(sScene* scene,char* name)
{
	index_t ind = get_index_by_name(name,scene->materials,scene->material_count);
	return ind==0xFFFFFFFF ? 0 : &scene->materials[ind];
}

void sSceneAppendObject(GameObject* obj)
{
	sScene* scene = obj->object.scene;
	//printf("sSceneAppendObject(%s)\n{\n",obj->object.name);
	switch (obj->object.name[0])
	{
		case 'o' :
		{
			//printf("\t%d\n",sizeof(void*)*scene->objects_count+sizeof(void*));
			scene->objects = sRealloc(scene->objects,sizeof(void*)*(scene->objects_count+1));
			scene->objects[scene->objects_count++] = (sObject*)obj;break;
		}
		case 's' :
		{
			//printf("\t%d\n",sizeof(void*)*scene->skelets_count+sizeof(void*));
			scene->skeletons = sRealloc(scene->skeletons,sizeof(void*)*(scene->skeletons_count+1));
			scene->skeletons[scene->skeletons_count++] = (sSkeleton*)obj;break;
		}
		case 'l' :
		{
			//printf("\t%d\n",sizeof(void*)*scene->lights_count+sizeof(void*));
			scene->lights = sRealloc(scene->lights,sizeof(void*)*(scene->lights_count+1));
			scene->lights[scene->lights_count++] = (sLight*)obj;break;
		}
	}
	//printf("\t%d\n}\n",sizeof(void*)*scene->gobjects_count+sizeof(void*));
	scene->gobjects = sRealloc(scene->gobjects,(scene->gobjects_count+1)*sizeof(void*));
	scene->gobjects[scene->gobjects_count++] = obj;
	obj->object.hash = scene->gobjects_unique_counter++;
}

sObject* sSceneAddObject(sScene* scene,char* name)
{
	uint32_t ind;
	switch (name[0])
	{
		case 'l' :
		{
			ind = get_index_by_name(name,scene->lights_inactive,scene->lights_inactive_count);
			if (ind!=0xFFFFFFFF)
			{
				return sObjectDuplicate(&scene->lights_inactive[ind]);
			}
			break;
		}
		case 'o' :
		{
			ind = get_index_by_name(name,scene->objects_inactive,scene->objects_inactive_count);
			if (ind!=0xFFFFFFFF)
			{
				return sObjectDuplicate(&scene->objects_inactive[ind]);
			}
			break;
		}
		case 's' :
		{
			ind = get_index_by_name(name,scene->skelets_inactive,scene->skelets_inactive_count);
			if (ind!=0xFFFFFFFF)
			{
				return sObjectDuplicate(&scene->skelets_inactive[ind]);
			}
			break;
		}
	}
	fprintf(stderr,"Warning: %s not such object\n",name);
	return 0;
}

void printChildren(void* obj,index_t level)
{
	sObject* object = obj;
	if (level>1000)
	{
		fprintf(LOGOUT, "Children loop detected\n");
		exit(-1);
	}
	//printf("%s %d\n",object->name,object->child_count);
	for (uint32_t i=0;i<object->child_count;i++)
	{
		//printf("%s \n",((sObject*)object->children[i])->name);
		for (index_t n=0;n<level;n++)
		{
			fprintf(LOGOUT, "\t");
		}
		fprintf(LOGOUT, "%p %s (%s) parent %p\n",
				object->children[i],
				((sObject*)object->children[i])->name,
				(isDuplicate(object->children[i]) ? "duplicate" : "inactive"),
				((sObject*)object->children[i])->parent);
		printChildren(object->children[i],level+1);
	}
}

void sScenePrintObjects(sScene* scene)
{
	fprintf(LOGOUT, "\nAll scene objects:\n");
	for (index_t i=0;i<scene->gobjects_count;i++)
	{
		sObject* obj = (sObject*)scene->gobjects[i];
		if (obj->parent) continue;
		fprintf(LOGOUT, "%p %s (%s)\n",obj,obj->name,isDuplicate(obj) ? "duplicate" : "inactive");
		printChildren(obj,1);
	}
	/*
	fprintf(LOGOUT, "\n");
	for (index_t i=0;i<scene->objects_inactive_count;i++)
	{
		sObject* obj = &scene->objects_inactive[i];
		if (obj->parent) continue;
		fprintf(LOGOUT, "0x%08lX %s (%s)\n",(intptr)obj,obj->name,isDuplicate(obj) ? "duplicate" : "inactive");
		printChildren(obj,1);
	}
	for (index_t i=0;i<scene->lights_inactive_count;i++)
	{
		sLight* obj = &scene->lights_inactive[i];
		if (obj->parent) continue;
		fprintf(LOGOUT, "0x%08lX %s (%s)\n",(intptr)obj,obj->name,isDuplicate(obj) ? "duplicate" : "inactive");
		printChildren(obj,1);
	}
	for (index_t i=0;i<scene->skelets_inactive_count;i++)
	{
		sSkeleton* obj = &scene->skelets_inactive[i];
		if (obj->parent) continue;
		fprintf(LOGOUT, "0x%08lX %s (%s)\n",(intptr)obj,obj->name,isDuplicate(obj) ? "duplicate" : "inactive");
		printChildren(obj,1);
	}*/
}

void sSceneDraw(sScene* scene)
{
	GLuint material=3;
	fprintf(LOGOUT, "Setting uniforms for \n");
	for (index_t i=0;i<scene->material_count;i++)
	{
		if (material==scene->materials[i].shader) continue;
		material = scene->materials[i].shader;
		useProgram(material);
		fprintf(LOGOUT, "%d\n",material);

		sMaterialUniformLA(material,"camera",&scene->camera.transform);
		sMaterialUniformiv(material,"lights_count",&(scene->lights_inactive_count),1);
		for (index_t light=0;light<scene->lights_inactive_count;light++)
		{
			sMaterialStructLA(material,"lights","transform",i,&scene->lights_inactive[i].transform);
			sMaterialStructiv(material,"lights","type",i,&scene->lights_inactive[i].type,1);
			sMaterialStructfv(material,"lights","inner",i,&scene->lights_inactive[i].inner,1);
			sMaterialStructfv(material,"lights","outer",i,&scene->lights_inactive[i].outer,1);
			sMaterialStructfv(material,"lights","color",i,&scene->lights_inactive[i].color,4);
		}
	}
}

void sSceneSetScript(sScene* scene,uint32_t (script)(sObject*))
{
	scene->behaviour = (fptr)script;
}

void sSceneSetSkyTexture(sScene* scene,sTexture* cubemap)
{
	if (scene && cubemap)
	{
		scene->cubemap = cubemap;
	}
}

void sSceneDefragLists(sScene* scene)
{
	void defrag_list(void** list, uint32_t* list_length)
	{
		int i=0, j=0;
		for ( ; i<*list_length; i++)
		{
			list[j] = list[i];
			if (list[i])
			{
				j++;
			}
		}
		*list_length = j;
	}
	defrag_list((void**)scene->gobjects, &scene->gobjects_count);
	defrag_list((void**)scene->objects, &scene->objects_count);
	defrag_list((void**)scene->lights, &scene->lights_count);
	defrag_list((void**)scene->skeletons, &scene->skeletons_count);
}

void sSceneFree(sScene* scene)
{
	if (!scene)
	{
		return;
	}
	sCameraDestroyFB(&scene->camera);

	while (scene->gobjects_count>1)
	{
		sObjectDelDuplicate(scene->gobjects[1]);
	}
	sPhysicsStop(scene);
	for (index_t i=0;i<scene->objects_inactive_count;i++)
	{
		if (scene->objects_inactive[i].children)
			sFree(scene->objects_inactive[i].children);
	}

	//fputs("Freeing active objects lists\n", LOGOUT);
	if (scene->gobjects) sFree(scene->gobjects);

	if (scene->skeletons) sFree(scene->skeletons);
	if (scene->objects) sFree(scene->objects);
	if (scene->lights) sFree(scene->lights);

	//fputs("Freeing skeleton\n", LOGOUT);
	for (index_t i=0;i<scene->skelets_inactive_count;i++)
	{
		sSkeleton* skel = &scene->skelets_inactive[i];
		for (index_t b=0;b<skel->child_count;b++)
		{
			sObjectDelParent(skel->children[b]);
		}
		for (index_t b=0;b<skel->bone_count;b++)
		{
			sObjectDelParent(&skel->bones[b]);
		}
		if (skel->children) sFree(skel->children);
		sFree(skel->bones);
		sFree(skel->pose);
	}

	//fputs("Freeing inactive objects lists\n", LOGOUT);
	if (scene->lights_inactive) sFree(scene->lights_inactive);
	if (scene->skelets_inactive) sFree(scene->skelets_inactive);
	if (scene->objects_inactive) sFree(scene->objects_inactive);
	//fputs("Freeing meshes\n", LOGOUT);
	for (index_t i=0;i<scene->mesh_count;i++)
	{
		if (scene->meshes[i].link_matrix) sFree(scene->meshes[i].link_matrix);
		if (scene->meshes[i].vertices) sFree(scene->meshes[i].vertices);
		if (scene->meshes[i].indices) sFree(scene->meshes[i].indices);
	}
	//fputs("Freeing skeleton actions\n", LOGOUT);
	for (index_t i=0;i<scene->actions_count;i++)
	{
		for (index_t c=0;c<scene->actions[i].channels_count;c++) sFree(scene->actions[i].channels[c].keyframes);
		if (scene->actions[i].channels)
		{
			sFree(scene->actions[i].channels);
		}
	}
	//fputs("Freeing mesh buffers\n", LOGOUT);
	for (uint32_t i=0;i<scene->mesh_count;i++)
	{
		sMeshDeleteBuffers(scene->meshes + i);
	}
	//fputs("Freeing textures\n", LOGOUT);
	for (uint32_t i=0;i<scene->texture_count;i++)
	{
		sTextureFree(scene->textures + i);
	}

	if (scene->textures) sFree(scene->textures);
	if (scene->materials) sFree(scene->materials);
	if (scene->meshes) sFree(scene->meshes);
	if (scene->actions) sFree(scene->actions);

	if (scene == active_scene)
	{
		active_scene = 0;
		puts("active_scene = 0;");
	}
}
