/*
 * engine.h
 *
 *  Created on: 22 дек. 2017 г.
 *      Author: ivan
 */

#ifndef ENGINE_H_
#define ENGINE_H_

#define dDOUBLE
#define GLEW_STATIC

#include "GL/glew.h"
#include <GLFW/glfw3.h>
#include <ode/common.h>
#include <ode/ode.h>


#include <dirent.h>
#include <string.h>
#include <stdlib.h>
#include <math.h>

#include "linalg.h"
#include "sound.h"
#include "configs.h"


//#define DRAW_BONES
//#define glc(func) func
#define glc(func) {func;GLenum err=glGetError(); if (err) printf("GLerror:%s:%d : %d\n",__FILE__,__LINE__,err);}

// Check windows
#if _WIN32 || _WIN64
#if _WIN64
typedef uint64_t intptr;
#else
typedef uint32_t intptr;
#endif
#endif

// Check GCC
#if __GNUC__
#if __x86_64__ || __ppc64__
typedef uint64_t intptr;
#else
typedef uint32_t intptr;
#endif
#endif

#define range(var,start,end,step) for (uint32_t var=start;var<end;var+=step)
#define BUFFER_OFFSET(i) ((void*)(i))
void readf(void* data_ptr,uint32_t size, uint32_t count, FILE* file_ptr);
#define MAX(a,b) ((a)>(b) ? (a) : (b))
#define MIN(a,b) ((a)<(b) ? (a) : (b))

#define LOGGING
#define S_POINT 0
#define S_DIRECT 1
#define S_SUN 1
#define S_SPOT 2
#define RESDIR "data"
#define RESMESH ".mesh"
#define ACTIONRES ".anim"

#define LOGOUT stdout
#define ERROUT stderr

#define S_SPOT_SHADOW_SIZE 256.0
#define S_SUN_SHADOW_SIZE 2048.0

#define OBJECT_BASE uint32_t hash;char name[256];
#define OBJECT_TEMPLATE OBJECT_BASE index_t child_count;void* parent;void** children;void* scene;fptr behaviour;laType transform;laType transform_global;laType transform_global_previous;_Bool transformable;_Bool inactive;_Bool hidden;void* data;

#define CAMERA_TEMPLATE laType projection;laType viewProjection;\
	GLuint noise, render_texture,render_result,render_texture1,render_texture2,render_normal,render_normal_glass,render_specular,render_ambient,render_vectors,\
	render_depth, voxel_render_buffer, render_fb;\
	GLuint voxel_texture, voxel_frame_buffer, reserved2, reserved3;\
	sMesh render_plane;sShader* filters[8];uint32_t mipmap_layers;float zNear,zFar,FOV;uint16_t width;uint16_t height;\
	sObject *view_point;

typedef void(*fptr)(void*);
typedef uint32_t index_t;
typedef _Bool bool;
typedef unsigned long long size_s;

extern char buff[1024];
extern GLuint activeShader;
extern _Bool _renderVectors;
extern _Bool _renderHDR;
extern _Bool _renderBloom;
extern _Bool _renderRayTracing;
extern _Bool _renderDeferred;
extern _Bool _VXGI;
extern char shader_log_buffer[100000];

_Bool useProgram(GLuint id);

//stack.c
typedef struct
{
	uint32_t keys[64];
	uint8_t count;
} sKeyStack;

void sStackPol(sKeyStack* stack,uint32_t key);
uint16_t sStackGetState(sKeyStack* stack,uint32_t key);
////////////

/////String////
void sStringUnicodeTo1251(char* result, char* string);
///////////////

/////Memory////
void* sRealloc(void*,size_s);
void* sRecalloc(void*,size_s);
void* sMalloc(size_s);
void* sCalloc(size_s,size_s);
void sFree(void*);
size_s sSizeof(void* ptr);
size_s sGetAllocatedMem(void);
///////////////

////Sensors////
typedef enum
{
	rayX = 0b001,
	rayY = 0b010,
	rayZ = 0b011,
	rayXn = 0b101,
	rayYn = 0b110,
	rayZn = 0b111
} sRayDir;

typedef struct
{
	void* object;
	dReal position[3];
	dReal normal[3];
} sPhysicsContact;

typedef struct
{
	dSpaceID space;
	sPhysicsContact *contacts;
	index_t contactsCount;
	index_t contactsAllocated;
} sPhysicsCS;

typedef struct
{
	dSpaceID space;
	sPhysicsContact *contacts;
	index_t contactsCount;
	index_t contactsAllocated;
	dReal angle;
	dReal range;
	sRayDir dir;
	dGeomID radar_mesh;
} sPhysicsRS;
///////////////

typedef struct
{
	char name[256];
	uint32_t hash;
} S_NAME;

typedef struct
{
	S_NAME* names;
	uint32_t count;
} S_NAME_ARRAY;

void getFileList(char* dir_name,S_NAME_ARRAY* names);

//shader.c
typedef struct
{
	char name[256];
	GLchar* fragment_source;
	GLchar* vertex_source;
	GLsizei log_len;
	GLsizei frag_source_len;
	GLsizei vert_source_len;
	GLint success;
	GLuint compute;
	GLuint fragment;
	GLuint vertex;
	GLuint program;
	GLuint log;
	FILE* fp;
} sShader;

void sShaderSetVersion(char* version);
void sLoadFragmentFromFile(sShader* shader,const char*);
void sLoadVertexFromFile(sShader* shader,const char*);
void sShaderCompileFragment(sShader*);
void sShaderCompileVertex(sShader*);
void sShaderMake(sShader*);
void sShaderCompileMake(sShader*);
void sShaderCompileMakeFiles(sShader*,char*,char*);
void sShaderValidate();
void sShaderDestroy(sShader* shader);

//mesh.c
typedef struct
{
	float r,g,b,a;
} sColour;

typedef struct
{
	uint32_t hash;
	char name[256];
	uint16_t width;
	uint16_t height;
	void* data;
	uint32_t type;
	GLuint ID;
} sTexture;

typedef struct
{
	uint32_t hash;
	char name[256];
	dReal friction;
	float transparency;
	_Bool glass;
	float height_scale;
	sColour diffuse;
	sColour specular;
	sTexture* diffuse_texture;
	sTexture* specular_texture;
	sTexture* height_texture;
	sTexture* lightmap_texture;
	sTexture* reflection_cubemap;
	float tdx;
	float tdy;
	float glow;
	float wet;
	GLuint shader;
} sMaterial;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
} sVertex;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
	uint32_t w1,w2,w3;
} bVertex;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
	uint32_t u2,v2;
} smVertex;

typedef struct
{
	float vx,vy,vz;
	float nx,ny,nz;
	float u,v;
	float bx,by,bz;
	float tx,ty,tz;
	uint32_t w1,w2,w3;
	uint32_t u2,v2;
} smbVertex;

typedef struct
{
	sVertex vert[3];
} sTriangle;

typedef struct
{
	uint32_t hash;
	char name[256];
	laType* link_matrix;
	sVertex* vertices;
	index_t* indices;
	uint32_t ind_count;
	uint32_t vert_count;
	sMaterial* material;
	GLuint VBO;
	GLuint IBO;
	GLuint uniforms[4];
	laType transform;
	laType bounding_box;
	uint16_t bones_indices[128];
	_Bool deformed;
	_Bool uv2;
	void* owner;
} sMesh;

void S_Vertex_Calc_Normal(sVertex*);
void sMeshMakeBuffers(sMesh* mesh);
void sMeshDraw(sMesh* mesh);
void sMeshLoad(sMesh* mesh, sMaterial* mat,char* name);
void sMeshDelete(sMesh* mesh);
void sMeshDeleteBuffers(sMesh* mesh);

//Animation
#define MAX_ACTION_LAYERS 8
typedef enum
{
	ACTION_DUMMY = 0x00,
	ACTION_PLAY,
	ACTION_LOOP,
	ACTION_STOPED,
	ACTION_PLAY_ADD = 0x11,
	ACTION_LOOP_ADD,
	ACTION_STOPED_ADD,
	ACTION_PLAY_MIX = 0x01,
	ACTION_LOOP_MIX,
	ACTION_STOPED_MIX
} ACTION_TYPE;

typedef struct
{
	char name[256];
	laType* keyframes;
	laType  start_keyframe;
	index_t keyframes_count;
} sActionChannel;

typedef struct
{
	char name[256];
	index_t channels_count;
	float time;
	float time_begin;
	float time_end;
	float time_increment;
	ACTION_TYPE type;
	ACTION_TYPE blend_type;
	sActionChannel* channels;
	int* bones_matching;
} sAction;

// Skeletal actions //
typedef struct
{
	OBJECT_TEMPLATE;
	void* skeleton;
	sMesh* mesh;
	int animated;
	float length;
	int physicsID;
	int physicsType;
} sBone;

typedef struct
{
	OBJECT_TEMPLATE;
	index_t bone_count;
	sAction action[MAX_ACTION_LAYERS];
	float action_weights[MAX_ACTION_LAYERS];
	laType* pose;
	sBone* bones;
} sSkeleton;


// Обрабатывает аимацию скелета. Двигает кости согласно заданным на слоях анимациям.
void sActionProcess(sSkeleton*);

// Задаёт скелету анимацию по её имени
void sSkeletonSetAction(sSkeleton*,uint8_t layer,char*);

// Начинает воспроизведение анимации
// sActionPlay(скелет, слой, тип_воспроизведения, начальный_кадр, конечный_кадр, скорость)
void sActionPlay(sSkeleton* skeleton, uint8_t layer, uint32_t act_type, float start, float stop, float speed);

// Задаёт параметры (способ воспроизведения, начальный и конечный кадры и скорость)
// sActionSetParam(скелет, слой, тип_воспроизведения, начальный_кадр, конечный_кадр, скорость)
void sActionSetParam(sSkeleton* skeleton, uint8_t layer, int32_t act_type, float start, float stop, float speed);

// Задаёт анимацию и её слой, способ воспроизведения, интервал и скорость.
void sSkeletonSetPlayAction(sSkeleton* skel,char* name,uint8_t layer, uint32_t act_type, float start, float stop, float speed);

// Задаёт кадр анимации, воспроизводящейся на заданном слое
void sSkeletonSetActionFrame(sSkeleton* skel, uint8_t layer,float frame);

// Проверяет, воспроизводится ли на заданном слое анимация
// sActionStop(скелет, слой)
uint8_t sActionIsPlaying(sSkeleton* skeleton,uint8_t layer);

// Останавливает анимацию, воспроизводящуюся на заданном слое
// sActionStop(скелет, слой)
void sActionStop(sSkeleton* skeleton,uint8_t layer);

// Возвращает кость скелета по её имени
sBone* sSkeletonGetBone(sSkeleton* skeleton,char* name);

// Возвращает индекс ксти скелета
sBone* sSkeletonGetBoneByIndex(sSkeleton* skeleton,uint16_t ind);

// Возвращает количество костей скелета
uint16_t sSkeletonGetBoneCount(sSkeleton* skeleton);

// Возвращает скелет, которму принадлежит кость
sSkeleton* sBoneGetSkeleton(sBone*);

// Возвращает флаг анимации кости
int sBoneGetAnimatedFlag(sBone*);

// Включает или отключает анимацию кости
void sBoneSetAnimatedFlag(sBone*,int);

// Возвращает кадр анимации, воспроизводящейся на задонном слое
float sSkeletonGetActionFrame(sSkeleton* skeleton,int layer);

// Задаёт фазу анимации
void sSkeletonSetActionFrame2(sSkeleton* skel, uint8_t layer, float coeff);

// Возвращает фазу анимации
float sSkeletonGetActionFrame2(sSkeleton* skel, uint8_t layer);

// Меняет слои анимации местами
void sSkeletonSwapLayers(sSkeleton* skeleton, uint16_t a, uint16_t b);

// Задаёт вес слоя анимации. Вес - коэффициент смешения анимационного слоя с предыдущим
void sSkeletonSetLayerWeight(sSkeleton* skel, uint8_t layer, float weight);
void sSkeletonSetLayerTime(sSkeleton* skel, uint8_t layer, float time);
float sSkeletonGetLayerTime(sSkeleton* skel, uint8_t layer);
void sSkeletonSetLayerSpeed(sSkeleton* skel, uint8_t layer, float speed);
float sSkeletonGetLayerSpeed(sSkeleton* skel, uint8_t layer);
void sSkeletonSetActionInterval(sSkeleton* skeleton, uint8_t layer, float a, float b);
void sSkeletonResetPose(sSkeleton* skeleton);
void sSkeletonAddPoseFromLayerToPose(sSkeleton* skeleton, uint8_t layer, float time, float weight);
void sSkeletonMixPoseFromLayerWithPose(sSkeleton* skeleton, uint8_t layer, float time, float weight);
void sSkeletonAddPoseFromActionToPose(sSkeleton* skeleton, char* name, uint32_t keyframe, float time, float weight);
void sSkeletonMixPoseFromActionWithPose(sSkeleton* skeleton, char* name, uint32_t keyframe, float time, float weight);
///////////////////////

typedef struct
{
	OBJECT_TEMPLATE;
	sSkeleton* skeleton;
	sMesh* mesh;
	dBodyID body;
	dBodyID bodyIgnoring;
	dGeomID geom;
	sPhysicsRS radar;\
	sPhysicsRS ray;\
	sPhysicsCS collider;
	int ghost;
	int physics_enabled;
	//dGeomID rays[6];
	uint32_t physicsType;
	uint32_t physicsShape;
	uint32_t collisionGroups;
	uint32_t collideWithGroups;
	dReal physicsFriction;
	dReal physicsMass;
	dReal averangeVel;
} sObject;

typedef struct
{
	OBJECT_TEMPLATE;
	CAMERA_TEMPLATE;
} sCamera;

typedef struct
{
	OBJECT_TEMPLATE;
	CAMERA_TEMPLATE;
	uint8_t type;
	sColour color;
	float inner,outer;
	bool shadow;
} sLight;

void sObjectPrintChildren(void* obj);
void sObjectMoveGlobal3fv(void* obj,laType vector);
void sObjectSetPositionLocal3fv(void* obj,laType vector);
void sObjectSetPositionGlobal3fv(void* obj,laType vector);
void sObjectSetLocalTransform(void* obj,laType transform);
void sObjectMoveLocal3fv(void* obj,laType vector);
void sObjectRotateLocal3f(void* object,float x,float y,float z);
void sObjectRotateGlobal3f(void* object,float x,float y,float z);
void sObjectSetRotation3f(void* object,float x,float y,float z);
laType sObjectGetVectorTo(void* object,void* object_to);
float sObjectGetDistanceTo(void* object,void* object_to);
laType sObjectGlobalTransform(sObject* object);
//void sObjectSnapToOther(void* object,void* other);

void sObjectPlaceChildren(sObject* object);
void* sObjectDuplicate(void* obj);
void* sObjectDuplicate2(void* obj);

void sCameraTakeScreenshot(sCamera* camera, char* file_name);
void sCameraInitShadowFB(sCamera* camera);
void sCameraAddFilter(sCamera* camera,char* file);
void sCameraAddPPShader(sCamera* camera,sShader* shader);
uint8_t sCameraGetFiltersCount(sCamera* camera);
void sCameraLoadSkyShader(sCamera* camera, char* file);
void sCameraInitFB(sCamera* camera);
void sCameraDestroyFB(sCamera* camera);

void sRenderInitVoxel(int size);
void sRenderTo3DTexture(sCamera* camera);

laType sCameraGetTransform(sCamera* camera);
laType sCameraGetTransformPrev(sCamera* camera);
laType *sCameraGetTransformLink(sCamera* camera);
laType *sCameraGetTransformPrevLnk(sCamera* camera);
void sRenderShading(sCamera* camera);
void sCameraBindVectorsFB(sCamera* camera);
void sCameraBindFB(sCamera* camera, int transparency);

sTexture* sObjectGetCubemap(sObject* obj);
void sObjectSetCubemap(sObject* obj, sTexture* cubemap);

uint32_t sObjectGetChildCount(sObject* object);
sObject* sObjectGetChildren(sObject* object,uint32_t ind);
sObject* sObjectGetChild(sObject* object,char* name);
sObject* sObjectGetParent(sObject* object);

void sObjectSetParent(void* obj,void* parent,_Bool apply_transform);
void sObjectDelParent(void* obj);
void sObjectRemoveParent(void* obj);
void sObjectPrintHierarchy(void* obj, int recursion);
_Bool isDuplicate(void* obj);
void sObjectSetMaterialFriction(sObject* obj,float friction);
void sMaterialUniformLA(GLuint mat,char* name,laType* data);
void sMaterialTexture(GLuint id,char* name,uint32_t ID,GLint index);
void sMaterialTextureID(GLuint id,char* name,sTexture* ID,GLint index);
void sMaterialUniformfv(GLuint mat,char* name,void* data,GLuint count);
void sMaterialUniformiv(GLuint mat,char* name,void* data,GLuint count);
void sMaterialStructLA(GLuint mat,char* name,char* attr,int index,laType* data);
void sMaterialStructfv(GLuint mat,char* name,char* attr,int index,void* data,uint32_t count);
void sMaterialStructiv(GLuint mat,char* name,char* attr,int index,void* data,uint32_t count);
void sMaterialUniformf(GLuint,char*,float);
void sMaterialUniformi(GLuint,char*,long);
void sObjectDelDuplicate(void*);
void sObjectIK(void* obj,void* elb,void* tar);

typedef union
{
	sObject object;
	sLight light;
	sCamera camera;
	sSkeleton skeleton;
	sBone bone;
} GameObject;


//scene.c
typedef struct
{
	void (*behaviour)(void*);
	sCamera camera;

	sMesh* meshes;
	sMaterial* materials;
	sTexture* textures;
	sTexture* cubemap;
	sLight** lights;
	sObject** objects;
	sSkeleton** skeletons;
	sLight* lights_inactive;
	sObject* objects_inactive;
	sSkeleton* skelets_inactive;
	sAction* actions;

	GameObject** gobjects;
	index_t gobjects_count;
	index_t gobjects_unique_counter;

	sShader *shader_list[8];

	index_t mesh_count;
	index_t material_count;
	index_t texture_count;
	index_t lights_count;
	index_t objects_count;
	index_t skeletons_count;
	uint32_t lights_inactive_count;
	index_t objects_inactive_count;
	index_t skelets_inactive_count;
	index_t actions_count;
	dWorldID world;
	dSpaceID space;
	dJointGroupID contactgroup;
	dJointGroupID joints;
} sScene;
extern sScene SCENE;


// Renderer //
void sRenderSwapPPShaders(void);

void sRenderLoadShaders(void);
void sRenderDestroyShaders(void);

void sRenderRayTracingOff(void);
void sRenderRayTracingOn(void);
void sRenderVectorsOff(void);
void sRenderVectorsOn(void);
void sRenderBloomOn(void);
void sRenderBloomOff(void);

void sRenderSetReflections(int val);
int  sRenderGetReflections(void);
void sRenderSetSSGI(int val);
int  sRenderGetSSGI(void);
void sRenderSetMotionBlur(int val);
int  sRenderGetMotionBlur(void);
void sRenderSetHDR(int val);
int  sRenderGetHDR(void);
void sRenderSetBloom(int val);
int  sRenderGetBloom(void);

void sRenderClear(float r,float g, float b, float a);
void sRenderDrawObject(sObject* object,sCamera* camera, _Bool cull_invisible);
void sRenderDrawScene(sScene *scene);
sMesh* sSceneAddMesh(sScene* scene, char* mesh);
void sSceneRemoveMesh(sScene* scene, char* name);
/////////////

// Skeletal actions //
void sActionLoad(sScene* scene,char* path,char* name);
//////////////////////

void sSceneDefragLists(sScene* scene);
void sSceneDraw(sScene* scene);
void* sSceneGetObject(sScene* scene,char* name);
sMaterial* sSceneGetMaterial(sScene* scene,char* name);
uint32_t S_Name2hash(char* name);
void sSceneLoad(sScene* scene,char* name);
void sSceneSetSkyTexture(sScene* scene,sTexture* cubemap);
void sScenePrintObjects(sScene* scene);
void sSceneAppendObject(GameObject* obj);
void sSceneFunctionLoop(sScene* scene);
void sSceneFunctionSet(sScene* scene);
void printChildren(void* obj,index_t level);
void sSceneFree(sScene* scene);

//engine.c
char *sGetProfilingString(void);
int sEngineGetWidth(void);
int sEngineGetHeight(void);
void sEngineStartLoop(void);
void sEngineSwapBuffers(void);
void sEngineStartOpenGL(void);
void sEngineSetActiveScene(sScene* scene);
sScene* sEngineGetActiveScene(void);
void sEngineClose(void);

sScene* sLoadScene(char* name);
void game(sScene* scene);

double sGetFrameTime(void);
_Bool sGetInputTick(void);

int gflwGetMbutton(int bttn);
void sMouseSetPosition(float x,float y);
void sMouseGetPosition(float* x,float* y);
void sMouseGetDelta(float* x,float* y);
int sKeyboardGetKeyState(int key);
int sMouseGetKeyState(int key);
float sMouseGetVerticalScroll(void);
float sMouseGetHorizontalScroll(void);
//////////

//physics.c

void sPhysicsApplyForceAtPointGlobal3fv(sObject* obj,laType pos,laType vec);
void sPhysicsApplyImpulseAtPointGlobal3fv(sObject* obj,laType pos,laType vec);
void sPhysicsApplyHitAtPointGlobal3fv(sObject* obj,laType pos,laType vec,float mass);

void sPhysicsInit(sScene* scene);
void sPhysicsStep(sScene* scene,double);
void sPhysicsStop(sScene* scene);
void sPhysicsAttach(sObject* object);

void sPhysicsSetAngularVelocity(sObject* object, double x, double y, double z);

void sPhysicsSetSpeedGlobal(void* object,laType vel,uint8_t axes);
void sPhysicsSetSpeedXLocal(void* object,float vel);
void sPhysicsSetSpeedYLocal(void* object,float vel);
void sPhysicsSetSpeedZLocal(void* object,float vel);


void sPhysicsCSInit(sObject* object);
void sPhysicsRSInit(sObject* object,float range);
void sPhysicsRadSInit(sObject* object,float range,float angle);

void sPhysicsCSAddContact(sObject* object,sObject* obj,dReal* position,dReal* normal);
void sPhysicsRSAddContact(sObject* object,sObject* obj,dReal* position,dReal* normal);
void sPhysicsRadSAddContact(sObject* object,sObject* obj,dReal* position,dReal* normal);

void sPhysicsCSClear(sObject* object);
void sPhysicsRSClear(sObject* object);
void sPhysicsRadSClear(sObject* object);

void sPhysicsCSFree(sObject* object);
void sPhysicsRSFree(sObject* object);
void sPhysicsRadSFree(sObject* object);

laType sPhysicsRSGetHitNormal(sObject* obj);
laType sPhysicsRSGetHitPosition(sObject* obj);

index_t sPhysicsCSGetHitObjectCount(sObject* obj);
sObject* sPhysicsCSGetHitObject(sObject* obj,index_t ind);

index_t sPhysicsRadSGetHitObjectCount(sObject* obj);
sObject* sPhysicsRadSGetHitObject(sObject* obj,index_t ind);

index_t sPhysicsRSGetHitObjectCount(sObject* obj);
sObject* sPhysicsRSGetHitObject(sObject* obj,index_t ind);

void sPhysicsResume(sObject* object);
void sPhysicsSuspend(sObject* object);

dJointID sPhysicsCreateAnchor(sObject* obj1,sObject* obj2,dReal minAngle,dReal maxAngle,
		laType axis_pos_relative,laType axis_dir, _Bool relative);
dJointID sPhysicsCreateBallSocket(sObject* obj1,sObject* obj2,laType anch_pos_relative, _Bool relative);
dJointID sPhysicsCreateCardan(sObject* obj1,sObject* obj2,laType anch_pos_relative,laType axis1,laType axis2, _Bool relative);

double sPhysicsJointGetAngle1(dJointID joint);
double sPhysicsJointGetAngle2(dJointID joint);
double sPhysicsJointGetAngle1Rate(dJointID joint);
void sPhysicsJointSetAngle1Rate(dJointID joint, double vel, double force);
double sPhysicsJointGetAngle2Rate(dJointID joint);
void sPhysicsJointSetAngle2Rate(dJointID joint, double vel, double force);
int sPhysicsJointGetAxisCount(dJointID joint);
///////////

////Utils////
typedef struct
{
	uint32_t format;
	uint32_t signature;
	uint32_t height;
	uint32_t width;
	uint32_t mipMapNumber;
	uint32_t formatCode;
	uint32_t blockSize;
	uint32_t offset;
	void *dataPtr;
} DDS_DATA;

int sTextureLoadDDSFromString(sTexture* texture, char* content);
int sTextureLoadDDS(sTexture* texture,char* name);
int sTextureLoadCubemap(sTexture* texture,char* name);
void sTextureGenerateBlueNoise(sTexture* texture);
void sTextureFree(sTexture *texture);
/////////////

////python api////
typedef struct
{
	float x,y;
} coordinates;
void sGetMousePosition(float*);
coordinates sGetMouseDelta(void);
void sSetMousePosition(float,float);
void sEngineSetSwapInterval(uint32_t);
double sEngineGetTime(void);

sMesh* sSceneGetMesh(sScene*,char*);
sObject* sSceneAddObject(sScene*,char*);

char* sMeshGetName(sMesh*);
void sMeshSetMaterial(sMesh* mesh, sScene* scene, char* name);

laMatrix sObjectGetTransform(sObject*);
char* sObjectGetName(sObject*);

void sObjectInitDict(sObject* object);

void sObjectSetBehaviour(sObject* object, fptr func);
sScene* sObjectGetScene(void* object);
sMesh* sObjectGetMesh(sObject* object);
void sObjectSetMesh(sObject* object,sMesh* mesh);
void sObjectSetMeshByName(sObject* object,char* mesh);
void sObjectTrackToOther(sObject* obj1,sObject* obj2,uint8_t look_axis,uint8_t up_axis);
void sObjectTrackToPoint(sObject* obj1,laType point,uint8_t look_axis,uint8_t up_axis);

void sLightSetColor(void* light, float color[4]);

sSkeleton* sSkeleton_cast(void* obj);
sObject* sObject_cast(void* obj);
sLight* sLight_cast(void* obj);
sBone* sBone_cast(void* obj);
sCamera* sCamera_cast(sObject* obj);

laType sPhysicsGetLinearVelocity(sObject* obj);
laType sPhysicsRSGetHitNormal3f(sObject* obj,index_t ind);
laType sPhysicsRSGetHitPosition3f(sObject* obj,index_t ind);
laType sPhysicsCSGetHitNormal3f(sObject* obj,index_t ind);
laType sPhysicsCSGetHitPosition3f(sObject* obj,index_t ind);
laType sPhysicsRadSGetHitNormal3f(sObject* obj,index_t ind);
laType sPhysicsRadSGetHitPosition3f(sObject* obj,index_t ind);

void sPhysicsAutoDisable(sObject* obj,uint8_t flag);
void sPhysicsRSSetRange(sPhysicsRS* obj,float range);
void sPhysicsRadarSetAngle(sPhysicsRS* radar,float angle);

void sPhysicsRadSSetDirection(sPhysicsRS* radar,int dir);
//void sPhysicsRadSSetRange(sPhysicsRS* radar,float range);
//void sPhysicsRadSSetAngle(sPhysicsRS* radar,float angle);


//window.c
void sWindowDrawImage(char *data,uint16_t width,uint16_t height);
void sEngineCreateWindow(uint16_t width,uint16_t height,_Bool fullscreen);
int sEngineShouldClose(void);
//////////////////
#endif /* ENGINE_H_ */
