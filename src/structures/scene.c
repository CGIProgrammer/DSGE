/*
 * scene.c
 *
 *  Created on: 10 июля 2020 г.
 *      Author: Ivan G
 */

#include "structures/scene.h"
#include "structures/list.h"
#include "memmanager.h"
#include <string.h>
#include "io.h"

static sSceneID* scenes = 0;

sSceneID sSceneCreate(char* name)
{
    sSceneID result = sNew(sScene);
    strcpy(result->name, name);
    sListPushBack(scenes, result);
    return result;
}

void sSceneDelete(sScene* scene)
{
    sListPopItem(scenes, scene);
    size_t obj_count = sListGetSize(scene->objects);
    for (long i=obj_count-1; i>=0; i--)
    {
        printf("%d \n", (int)i);
        //printf("Deleting %s\n", scene->objects[i]->name);
        sGameObjectDelete(scene->objects[i]);
    }
    sDelete(scene);
}

void sSceneAddObject(sSceneID scene, sGameObjectID obj)
{
    if (!obj) return;
    sListPushBack(scene->objects, obj);
    obj->scene = scene;
}

void sSceneSetSkybox(sSceneID scene, sTextureID sky)
{
    if (sky) {
        scene->skybox = sky;
    }
}

void sSceneSetActiveCamera(sSceneID scene, sGameObjectID cam)
{
    if (cam && cam->camera_component) scene->camera = cam;
}

void sSceneAddBehaviourCallback(sSceneID scene, void (*behaviour)(sSceneID))
{
    if (behaviour) 
        sListPushBack(scene->behaviour, behaviour);
}

void sSceneClearEndedObjects(sScene* scene)
{
    for (size_t i=0; i<sListGetSize(scene->objects); i++)
    {
        if (scene->objects[i]->ended)
        {
            sGameObjectDelete(scene->objects[i]);
            i--;
        }
    }
}

void sSceneWorldStep(sSceneID scene, float delta)
{
    sSceneClearEndedObjects(scene);
    size_t obj_cnt = sListGetSize(scene->objects);
    for (size_t i=0; i<obj_cnt; i++)
    {
        scene->objects[i]->transform.global_prev_2 = scene->objects[i]->transform.global_prev_1;
        scene->objects[i]->transform.global_prev_1 = scene->objects[i]->transform.global;
    }
    for (size_t i=0; i<obj_cnt; i++)
    {
        if (!scene->objects[i]->parent)
        {
            sGameObjectApplyChildrenTransform(scene->objects[i]);
        }
    }
    
    size_t func_cnt = sListGetSize(scene->behaviour);
    for (size_t j=0; j<func_cnt; j++)
    {
        scene->behaviour[j](scene);
    }

    for (size_t i=0; i<obj_cnt; i++)
    {
        func_cnt = sListGetSize(scene->objects[i]->behaviour);

        for (size_t j=0; j<func_cnt; j++)
        {
            scene->objects[i]->behaviour[j](scene->objects[i]);
        }
    }
}

void sSceneDraw(sSceneID scene)
{
    if (scene->camera)
    {
        for (int i=0; i<sListGetSize(scene->objects); i++)
        {
            if (scene->objects[i]->light_component)
            {
                scene->objects[i]->light_component->rpclbk(scene->objects[i]->light_component, scene->objects);
            }
        }
        scene->camera->camera_component->rpclbk(scene->camera->camera_component, scene->objects, scene->objects, scene->skybox);
    }
}

static char* read_line(FILE* file)
{
    char* line = 0;
    char chr;
    for (chr=0; fread(&chr, 1, 1, file) && chr!='\n'; sListPushBack(line, chr));
    chr = 0;
    sListPushBack(line, chr);
    return line;
}

static char** split_string(char* string, char split_char, int max_count)
{
    char **parts = 0;
    char *part = 0;
    int i=0;
    char nl = 0;
    for (i=0;string[i]==split_char; i++);
    for (;string[i]; i++)
    {
        /*if (string[i]!=split_char || max_count==0)
        {
            sListPushBack(part, string[i]);
        }*/
        if (string[i]==split_char && string[i+1]!=split_char && max_count!=0)
        {
            sListPushBack(part, nl);
            sListPushBack(parts, part);
            part = 0;
            max_count--;
        } else {
            sListPushBack(part, string[i]);
        }
    }
    sListPushBack(part, nl);
    sListPushBack(parts, part);
    return parts;
}

static void string_lower_case(char* string)
{
    for (int i=0; string[i]; i++)
    {
        if (string[i]>='A' && string[i]<='Z')
        {
            string[i] += ('A' - 'a');
        }
    }
}

sSceneID sSceneLoadFromText(char* fname)
{
    //sSceneID scene = sSceneCreate(fname);
    sMaterialID material = 0;
    sTextureID texture = 0;
    sMeshID mesh = 0;
    sGameObjectID object = 0;

    sDict materials = sDictNew(20);

    int32_t mem = sGetAllocatedMem();
    FILE* fp = fopen(fname, "rb");
    for (char* line=0;;)
    {
        line = read_line(fp);
        if (strlen(line)==0)
        {
            sFree(line);
            break;
        }
        char** parts = split_string(line, ' ', 1);
        string_lower_case(parts[0]);

        if (!strcpy(parts[0], "material"))
        {
            material = sMaterialCreate(parts[1]);
            
        }
        for (int i=0; i<sListGetSize(parts); i++)
        {
            puts(parts[i]);
        }
        sFree2(parts);
        sFree(line);
    }

    printf("%d unfreed bytes of memory\n", sGetAllocatedMem()-mem);
}

sSceneID sSceneMakeDemo(void)
{
    sSceneID scene = sSceneCreate("Demoscene");

    puts("Create sky");
    sTextureID sky = sTextureCreateCubemap("Skybox", 512, 512, RGB16F, 1, 1);
    puts("Spliting cubemap");
    sTextureCubeSplit(sky);
    puts("Baking skybox");
    sCameraComponentBakeSkybox(sky);
    puts("Generating mipmaps");
    sTextureGenerateMipMaps(sky);
    puts("Setting skybox");
    sSceneSetSkybox(scene, sky);

    // Monkey and camera
    sGameObjectID monkey_obj = sGameObjectCreate((char*)"monkey");
    sMeshID monkey = sMeshLoad((char*)"data/mesh/monkey.mesh");
    sMaterialID material = sMaterialCreateWithDefaultShader((char*)"monkey_material");
    material->diffuse = (sColor){0.5,0.5,0.5,1};
    sMaterialSetDiffuseMap(material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/rustediron1-alt2-bl/rustediron2_basecolor.dds"));
    sMaterialSetHeightMap(material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/rustediron1-alt2-bl/rustediron2_normal.dds"));
    sMaterialSetMetallicMap(material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/rustediron1-alt2-bl/rustediron2_metallic.dds"));
    sMaterialSetRoughnessMap(material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/rustediron1-alt2-bl/rustediron2_roughness.dds"));
    material->metallic = 0.0;
    material->roughness = 0.02;
    sMeshSetMaterial(monkey, material);
    sGameObjectSetVisual(monkey_obj, monkey);
    monkey_obj->transform.global.a[11] = 1.0;
    ////////

    // Create plane
    sMeshID plane_mesh = sMeshLoad((char*)"data/mesh/plane.mesh");
    sGameObjectID plane_obj = sGameObjectCreate((char*)"plane");
    sMaterialID plane_material = sMaterialCreateWithDefaultShader((char*)"plane_material");
    sMaterialSetDiffuseMap(plane_material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/hardwood-brown-planks-bl/hardwood-brown-planks-albedo.dds"));
    sMaterialSetRoughnessMap(plane_material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/hardwood-brown-planks-bl/hardwood-brown-planks-roughness.dds"));
    sMaterialSetHeightMap(plane_material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/hardwood-brown-planks-bl/hardwood-brown-planks-normal-ogl.dds"));
    plane_material->metallic = 0.0;
    plane_material->roughness = 0.5;
    plane_material->diffuse = (sColor){1.0,1.0,1.0,1.0};
    sMeshSetMaterial(plane_mesh, plane_material);
    sGameObjectSetVisual(plane_obj, plane_mesh);
    //////

    sGameObjectID spotlight = sGameObjectCreate((char*)"spotlight");
    spotlight->transform.global = laRotationXYZ(radians(37.261), radians(3.16371), radians(106.936));
    spotlight->transform.global.a[3]  = 3;
    spotlight->transform.global.a[7]  = 1;
    spotlight->transform.global.a[11] = 4;
    spotlight->light_component = sLightCreateShadowBuffer(512, sLightSpot);
    spotlight->light_component->user = spotlight;
    spotlight->light_component->color = (sColor){
        15.0,
        15.0,
        15.0,1.0};

    sSceneAddObject(scene, plane_obj);
    sSceneAddObject(scene, monkey_obj);
    sSceneAddObject(scene, spotlight);

    return scene;
}
