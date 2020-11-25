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
extern int screen_width; // ширина окна
extern int screen_height; // высота окна

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
    if (!scene || delta<=0.0) return;
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
    if (scene && scene->camera)
    {
        for (size_t i=0; i<sListGetSize(scene->objects); i++)
        {
            if (scene->objects[i]->light_component && scene->objects[i]->light_component->rpclbk)
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
    if (line) {
        sListPushBack(line, chr);
    }
    return line;
}

/*static char** split_string(char* string, char split_char, int max_count)
{
    char **parts = 0;
    char *part = 0;
    int i=0;
    char nl = 0;
    for (i=0;string[i]==split_char; i++);
    for (;string[i]; i++)
    {
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
    sSceneID scene = sSceneCreate(fname);
    sMaterialID material = 0;
    sTextureID texture = 0;
    sMeshID mesh = 0;
    sGameObjectID object = 0;

    sDict materials = sDictNew(20);
    sDict meshes = sDictNew(20);

    int32_t mem = sGetAllocatedMem();
    FILE* fp = fopen(fname, "rb");
    for (char* line=0;;)
    {
        line = read_line(fp);
        if (!line) break;
        char** parts = split_string(line, ' ', 1);
        string_lower_case(parts[0]);

        if (!strcpy(parts[0], "material"))
        {
            sMaterialID material = sMaterialCreate(parts[1]);
            for (char* m_param=0; ;) {
                m_param = read_line(fp);
                char** param = split_string(m_param, ' ', 1);
                int params = 0;
                string_lower_case(param[0]);
                if (!strcmp(param[0], "diffuse")) {
                    sscanf( param[1], "%f %f %f%n", &material->diffuse.r, &material->diffuse.g, &material->diffuse.b, &params );
                    if (params!=3) {
                        material->diffuse_texture = sTextureLoad(param[1], param[1]);
                    }
                } else if (!strcmp(param[0], "metallic")) {
                    params = 0;
                    sscanf( param[1], "%f%n", &material->metallic, &params );
                    if (!params) {
                        material->metallic_texture = sTextureLoad(param[1], param[1]);
                    }
                } else if (!strcmp(param[0], "roughness")) {
                    params = 0;
                    sscanf( param[1], "%f%n", &material->roughness, &params );
                    if (!params) {
                        material->roughness_texture = sTextureLoad(param[1], param[1]);
                    }
                } else if (!strcmp(param[0], "normalmap")) {
                    params = 0;
                    material->height_texture = sTextureLoad(param[1], param[1]);
                } else if (!strcmp(param[0], "lightmap")) {
                    params = 0;
                    material->lightmap_texture = sTextureLoad(param[1], param[1]);
                } else if (!strcmp(param[0], "end") && !strcmp(param[1], "material")) {
                    sDictAddItemKW(&materials, material->name, (void*)material);
                }
                sFree(m_param);
                sFree2(param);
            }
        }
        if (!strcpy(parts[0], "mesh"))
        {
            sMeshID mesh = sMeshLoad(parts[1]);
            char* header = read_line(fp);
            char** param = split_string(header, ' ', 1);
            string_lower_case(param[0]);
            if (!strcmp(param[0], "material")) {
                mesh->material = (sMaterialID)param[1];
            }
            sFree(header);
            header = read_line(fp);
            string_lower_case(header);
            if (!strcmp(header, "end mesh")) {
                sDictAddItemKW(&meshes, mesh->name, mesh);
            }
            sFree(header);
            sFree2(param);
        }
        if (!strcpy(parts[0], "object"))
        {
            sGameObjectID object = sGameObjectCreate(parts[1]);
            float location[3] = {0, 0, 0};
            float rotation[3] = {0, 0, 0};
            for (char* m_param=0; ;) {
                m_param = read_line(fp);
                char** param = split_string(m_param, ' ', 1);
                int params = 0;
                string_lower_case(param[0]);
                if (!strcmp(param[0], "location")) {
                    sscanf( param[1], "%f %f %f%n", location, location+1, location+2, &params );
                    if (params!=3) {
                        printf(stderr, "Failed to set location of object (%s)", param[1]);
                        exit(-1);
                    }
                }
                if (!strcmp(param[0], "rotation")) {
                    sscanf( param[1], "%f %f %f%n", rotation, rotation+1, rotation+2, &params );
                    if (params!=3) {
                        printf(stderr, "Failed to set rotation of object (%s)", param[1]);
                        exit(-1);
                    }
                }
                sFree(m_param);
                sFree2(param);
            }
        }
        sFree2((void**)parts);
        sFree(line);
    }

    printf("%d unfreed bytes of memory\n", (int)(sGetAllocatedMem()-mem));
}*/

sGameObjectID load_camera(FILE* fp)
{
    char* name = read_line(fp);
    char* parent = read_line(fp);
    char* parent_bone = read_line(fp);
    float fov;
    sGameObjectID camera = sGameObjectCreate((const char*)name);
    readf((void*)camera->transform.local.a, sizeof(camera->transform.local.a), 1, fp);
    readf((void*)&fov, sizeof(fov), 1, fp);
    camera->camera_component = sCameraInitDeferredRenderer(screen_width, screen_height, 90.0, 1);
    camera->camera_component->user = camera;
    camera->parent = (sGameObjectID)parent;
    camera->transform.global = camera->transform.global_prev_1 = camera->transform.global_prev_2 = camera->transform.local;
    sFree(name);
    sFree(parent);
    sFree(parent_bone);
    return camera;
}

sMeshID pick_mesh(FILE* fp, sDict* meshes)
{
    char* name = read_line(fp);
    sMeshID tex = (sMeshID)sDictGetItemKW(meshes, name);
    sDelete(name);
    return tex;
}

sTextureID pick_texture(FILE* fp, sDict* textures)
{
    char* name = read_line(fp);
    if (name) {
        sTextureID tex = (sTextureID)sDictGetItemKW(textures, name);
        sDelete(name);
        return tex;
    } else {
        return 0;
    }
}

sGameObjectID load_light(FILE* fp)
{
    bool sas;
    uint8_t type;
    laMatrix transform;
    float color[4];
    float fov1, fov2, cs, ce;
    char* name = read_line(fp);
    puts(name);
    readf(&sas, 1, 1, fp);
    char* parent = read_line(fp);
    char* parent_bone = read_line(fp);
    sGameObjectID light = sGameObjectCreate((const char*)name);
    readf((void*)light->transform.local.a, sizeof(light->transform.local.a), 1, fp);
    readf((void*)color, sizeof(color), 1, fp);
    readf((void*)&type, 1, 1, fp);
    readf(&sas, 1, 1, fp);
    readf((void*)&fov1, 4, 1, fp);
    readf((void*)&fov2, 4, 1, fp);
    readf((void*)&cs, 4, 1, fp);
    readf((void*)&ce, 4, 1, fp);
    //type = sLightPoint;
    light->light_component = sLightCreateShadowBuffer(2048, type, 1);
    light->light_component->user = light;
    if (type==sLightSpot) {
        light->light_component->projection = laPerspective(2048, 2048, ce, cs, fov1*2 * 180.0 / 3.1415926535);
        light->light_component->projection_inv = laInverted(light->light_component->projection);
    }
    light->light_component->color.r = color[0] * color[3];
    light->light_component->color.g = color[1] * color[3];
    light->light_component->color.b = color[2] * color[3];
    light->parent = (sGameObjectID)parent;
    sFree(name);
    sFree(parent_bone);
    light->transform.global = light->transform.global_prev_1 = light->transform.global_prev_2 = light->transform.local;
    return light;
}

sGameObjectID load_object(FILE* fp, sMeshID meshes)
{
    uint8_t sas;
    uint32_t kuk;
    char* name = read_line(fp);
    sGameObjectID object = sGameObjectCreate(name);
    readf((void*)&object->hidden, 1, 1, fp);
    readf((void*)&sas, 1, 1, fp);
    object->hidden = object->hidden || sas;
    readf((void*)&sas, 1, 1, fp);
    char* parent = read_line(fp);
    char* parent_bone = read_line(fp);
    object->parent = (sGameObjectID)parent;
    sGameObjectSetVisual(object, pick_mesh(fp, meshes));
    readf((void*)object->transform.local.a, sizeof(object->transform.local.a), 1, fp);
    readf((void*)&kuk, 4, 1, fp);
    readf((void*)&kuk, 4, 1, fp);
    readf((void*)&kuk, 4, 1, fp);
    read_line(fp);
    sFree(name);
    sFree(parent_bone);
    object->transform.global = object->transform.global_prev_1 = object->transform.global_prev_2 = object->transform.local;
    return object;
}

sMaterialID load_material(FILE* fp, sDict* textures)
{
    char* name = read_line(fp);
    char dummy[8];
    sMaterialID material = sMaterialCreateWithDefaultShader(name);
    sDelete(name);
    material->diffuse_texture = pick_texture(fp, textures);
    sFree((void*)read_line(fp));
    material->roughness_texture = pick_texture(fp, textures);
    material->metallic_texture  = pick_texture(fp, textures);
    material->height_texture    = pick_texture(fp, textures);
    material->height_scale = material->height_texture != 0;
    pick_texture(fp, textures);

    readf((void*)&material->diffuse, sizeof(float), 3, fp);
    readf((void*)&material->specular, sizeof(float), 3, fp);
    readf((void*)&material->roughness, sizeof(float), 1, fp);
    readf((void*)&material->metallic, sizeof(float), 1, fp);
    readf((void*)&material->fresnel, sizeof(float), 1, fp);
    readf((void*)&material->glow, sizeof(float), 1, fp);
    readf((void*)dummy, sizeof(float), 1, fp);
    readf((void*)dummy, sizeof(double), 1, fp);
    //material->roughness = (1.0 - material->roughness) * (1.0 - material->specular.r);
    //material->metallic = 0.0;
    //material->glow = 0.0;
    return material;
}

sSceneID sSceneLoadBin(const char* fname)
{
    FILE *fp = fopen(fname, "rb");
    sSceneID scene = sSceneCreate(fname);
    sMeshID mesh = 0;
    sGameObjectID object = 0;
    sGameObjectID camera = 0;
    uint32_t count;

    sDict materials = sDictNew(20);
    sDict textures = sDictNew(20);
    sDict meshes = sDictNew(20);
    sDict objects = sDictNew(20);
    
    camera = load_camera(fp);
    sDictAddItemKW(&objects, camera->name, (void*)camera);
    sSceneSetActiveCamera(scene, camera);
    // Текстуры
    readf(&count, sizeof(count), 1, fp);
    for (uint32_t i=0; i<count; i++)
    {
        char* name = read_line(fp);
        char* filename = read_line(fp);
        char dds_name[512];
        sprintf(dds_name, "data/%s.dds", filename);
        sTextureID soap = sTextureLoadDDS(dds_name);
        if (soap) {
            glc(glBindTexture(soap->type, soap->ID));
            glc(glTexParameteri(soap->type, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
            glc(glTexParameteri(soap->type, GL_TEXTURE_MIN_FILTER, GL_LINEAR));
        }

        sDictAddItemKW(&textures, name, soap);
        sDelete(name);
        sDelete(filename);
    }

    // Материалы
    readf(&count, sizeof(count), 1, fp);
    for (uint32_t i=0; i<count; i++)
    {
        sMaterialID mat = load_material(fp, &textures);
        puts(mat->name);
        sDictAddItemKW(&materials, mat->name, mat);
    }

    // Модельки
    readf(&count, sizeof(count), 1, fp);
    for (uint32_t i=0; i<count; i++)
    {
        char* name = read_line(fp);
        char* mat_name = read_line(fp);
        sMaterialID mat = (sMaterialID)sDictGetItemKW(&materials, mat_name);
        char fname[512];
        sprintf(fname, "data/mesh/%s.mesh", name);
        mesh = sMeshLoad(fname);
        sMeshSetMaterial(mesh, mat);
        sDictAddItemKW(&meshes, name, mesh);
        sDelete(name);
        sDelete(mat_name);
    }

    // Источники света
    readf(&count, sizeof(count), 1, fp);
    for (uint32_t i=0; i<count; i++)
    {
        object = load_light(fp);
        sDictAddItemKW(&objects, object->name, (void*)object);
        sSceneAddObject(scene, object);
    }

    // Видимые объекты
    readf(&count, sizeof(count), 1, fp);
    for (uint32_t i=0; i<count; i++)
    {
        object = load_object(fp, &meshes);
        if (object->hidden) {
            continue;
        }
        sDictAddItemKW(&objects, object->name, (void*)object);
        sSceneAddObject(scene, object);
    }

    for (size_t i=0; i<sListGetSize(scene->objects); i++)
    {
        char* name = scene->objects[i]->parent;
        if (name)
        {
            //sGameObjectID parent = (sGameObjectID)sDictGetItemKW(&objects, name);
            sDelete(scene->objects[i]->parent);
            //sGameObjectSetParent(scene->objects[i], parent);
        }
    }
    /*if (sDictHaveItemKW(&textures, "grass_1"))
    {
        sTextureID travka = sDictGetItemKW(&textures, "grass_1");
        glc(glBindTexture(travka->type, travka->ID));
	    glc(glTexParameteri(travka->type, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
	    glc(glTexParameteri(travka->type, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
    }
    if (sDictHaveItemKW(&textures, "tree_leafs"))
    {
        sTextureID travka = sDictGetItemKW(&textures, "tree_leafs");
        glc(glBindTexture(travka->type, travka->ID));
	    glc(glTexParameteri(travka->type, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
	    glc(glTexParameteri(travka->type, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR));
    }*/
    sDictDelete(&materials);
    sDictDelete(&textures);
    sDictDelete(&meshes);
    sDictDelete(&objects);
    return scene;
}

sSceneID sSceneMakeDemo(void)
{
    sSceneID scene = sSceneCreate("Demoscene");

    puts("Create sky");
    sTextureID sky = sTextureCreateCubemap("Skybox", 2048, 2048, RGB16F, 1, 1);
    puts("Spliting cubemap");
    sTextureCubeSplit(sky);
    puts("Baking skybox");
    sCameraComponentBakeSkybox(sky);
    puts("Generating mipmaps");
    sTextureGenerateMipMaps(sky);
    //sky = sTextureLoadDDSCubemap("/home/ivan/SGM_SDK/SGE/data/textures/cubemap/flat.dds");
    puts("Setting skybox");
    sSceneSetSkybox(scene, sky);
    //sSceneSetSkybox(scene, sTextureLoadDDSCubemap("/home/ivan/SGM_SDK/SGE/data/textures/cubemap/cloudySea.dds"));

    // Monkey and camera
    sGameObjectID monkey_obj = sGameObjectCreate((char*)"monkey");
    sMeshID monkey = sMeshLoad((char*)"data/mesh/scifiengine.mesh");
    sMaterialID material = sMaterialCreateWithDefaultShader((char*)"monkey_material");
    material->diffuse = (sColor){1.0,1.0,1.0,1};
    sTextureID d = sTextureLoad((char*)"/home/ivan/Загрузки/3dexport__by_b_i_b_1530566435/Material _106_Base_Color.png", "dif");
    sTextureID n = sTextureLoad((char*)"/home/ivan/Загрузки/3dexport__by_b_i_b_1530566435/Material _106_Normal_OpenGL.png", "nor");
    sTextureID m = sTextureLoad((char*)"/home/ivan/Загрузки/3dexport__by_b_i_b_1530566435/Material _106_Metallic.png", "met");
    sTextureID r = sTextureLoad((char*)"/home/ivan/Загрузки/3dexport__by_b_i_b_1530566435/Material _106_Roughness.png", "rou");
    sTextureEnableMipmaps(d, 16);
    sTextureEnableMipmaps(n, 16);
    sTextureEnableMipmaps(m, 16);
    sTextureEnableMipmaps(r, 16);
    sMaterialSetDiffuseMap(material, d);
    sMaterialSetHeightMap(material, n);
    sMaterialSetMetallicMap(material, m);
    sMaterialSetRoughnessMap(material, r);
    material->metallic = 0.0;
    material->roughness = 1.0;
    material->height_scale = 1.0;
    sMeshSetMaterial(monkey, material);
    sGameObjectSetVisual(monkey_obj, monkey);
    monkey_obj->transform.global.a[11] = 0.4;
    ////////

    // Create plane
    sMeshID plane_mesh = sMeshLoad((char*)"data/mesh/plane.mesh");
    sGameObjectID plane_obj = sGameObjectCreate((char*)"plane");
    sMaterialID plane_material = sMaterialCreateWithDefaultShader((char*)"plane_material");
    sTextureID diffuse = sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/hardwood-brown-planks-bl/hardwood-brown-planks-albedo.dds");
    //sTextureID diffuse = sTextureLoad((char*)"/home/ivan/SGM_SDK/SGE/data/textures/levels/shooting_range/textures/sidewalk_col.jpg", "sidewalk_col");
    //sTextureID relief = sTextureLoad((char*)"/home/ivan/SGM_SDK/SGE/data/textures/sidewalk_relief.jpg", "sidewalk_relief");
    //sTextureEnableMipmaps(diffuse, 16);
    sMaterialSetDiffuseMap(plane_material, diffuse);
    //sMaterialSetHeightMap(plane_material, relief);
    sMaterialSetRoughnessMap(plane_material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/hardwood-brown-planks-bl/hardwood-brown-planks-roughness.dds"));
    sMaterialSetHeightMap(plane_material, sTextureLoadDDS((char*)"/home/ivan/SGM_SDK/SGE_2.0/data/textures/pbr/hardwood-brown-planks-bl/hardwood-brown-planks-normal-ogl.dds"));
    plane_material->metallic = 0.5;
    plane_material->roughness = 0.1;
    plane_material->diffuse = (sColor){1,1,1,1.0};
    plane_material->height_scale = 0.2;
    sMeshSetMaterial(plane_mesh, plane_material);
    sGameObjectSetVisual(plane_obj, plane_mesh);
    //////

    sGameObjectID spotlight = sGameObjectCreate((char*)"spotlight");
    spotlight->transform.global = laRotationXYZ(radians(65.261), radians(3.16371), radians(102.0));
    spotlight->transform.global.a[3]  = spotlight->transform.global.a[ 2] * 50.0;
    spotlight->transform.global.a[7]  = spotlight->transform.global.a[ 6] * 50.0;
    spotlight->transform.global.a[11] = spotlight->transform.global.a[10] * 50.0;
    spotlight->light_component = sLightCreateShadowBuffer(2048, sLightPoint, 1);
    spotlight->light_component->user = spotlight;
    spotlight->light_component->color = (sColor){
        10000.0,
        10000.0,
        10000.0,1.0};

    sSceneAddObject(scene, plane_obj);
    sSceneAddObject(scene, monkey_obj);
    sSceneAddObject(scene, spotlight);

    return scene;
}
