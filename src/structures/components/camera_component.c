#include "structures/framebuffer.h"
#include "structures/mesh.h"
#include "structures/components/light_component.h"
#include "structures/components/camera_component.h"
#include <unistd.h>
#include <SDL2/SDL.h>

#ifdef __cplusplus
extern "C" {
#endif

static sCameraComponentID* cameras = 0;
static sMeshID screen_plane = 0;
static sTextureID blue_noise;

static sShaderID sky_generator = 0;
static sFrameBuffer skybox_fb = {
    .color_render_targets = 0,
    .depth_render_target = 0,
    .framebuffer_id = 0,
    .renderbuffer_id = 0,
    .width = 0,
    .height = 0
};

void sCameraComponentBakeSkybox(sTextureID tex)
{
    if (!tex || !tex->ID || tex->type!=GL_TEXTURE_CUBE_MAP || !tex->sides) return;
    if (!screen_plane) screen_plane = sMeshCreateScreenPlane();
    if (!sky_generator) {
        sky_generator = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/sky_generator.glsl");
    }
    if (!skybox_fb.width && !skybox_fb.height)
    {
        skybox_fb = sFrameBufferCreate(tex->width, tex->height, 0);
        for (int i=0; i<6; i++)
        {
            skybox_fb.color_render_targets[i] = tex->sides[i];
        }
    }
    sTextureID* targets = skybox_fb.color_render_targets;
    laMatrix camera = laIdentity;
    laMatrix proj = laPerspective(tex->width, tex->height, 0.1, 100.0, 90.0);
    laMatrix projInv = laInverted(proj);
    sFrameBufferBind(&skybox_fb, 0b111111);

    sShaderBind(sky_generator);
    glc(sMeshDraw(screen_plane));
}

sCameraComponentID sCameraComponentCreate(uint16_t width, uint16_t height)
{
    sCameraComponentID camera = sNew(sCameraComponent);
    camera->framebuffer.width = width;
    camera->framebuffer.height = height;
    sListPushBack(cameras, camera);
    return camera;
}

void sCameraAddFilter(sCameraComponentID camera, sShaderID shader)
{
    if (camera && shader)
    {
        sListPushBack(camera->shaders, shader);
        sListPushBack(shader->render_users, camera);
    }
}

void sCameraPopFilter(sCameraComponentID camera, sShaderID shader)
{
    if (shader && camera)
    if (sListIndexOf(camera->shaders, shader) != MAX_INDEX)
    {
        sListPopItem(camera->shaders, shader);
        sListPopItem(shader->render_users, camera);
    }
}

void sCameraClearFilters(sCameraComponentID camera)
{
    size_t filter_count = sListGetSize(camera->shaders);
    for (size_t i=0; i<filter_count; i++)
    {
        sListPopItem(camera->shaders[i]->render_users, camera);
    }
    sDelete(camera->shaders);
}

static void sForwardRenderCallback(sCameraComponentID camera, sGameObjectID* draw_list, sGameObjectID* lights_list, sTextureID skybox)
{
    // Привязка буфера
    glc(sFrameBufferBind(&camera->framebuffer, 0));
    glc(glEnable(GL_DEPTH_TEST));
    glc(glDepthFunc(GL_LEQUAL));
    glc(glClearColor(0.2, 0.2, 0.2, 1));
    glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
    sMaterialBind(0);
    size_t objects_count = sListGetSize(draw_list);
    size_t lights_count = sListGetSize(lights_list);
    // Рендеринг объектов
    for (size_t i=0; i<objects_count; i++)
    {
        if (!draw_list[i]->visual_component) continue;
        sGameObjectID obj = draw_list[i];
        sMeshID mesh = obj->visual_component;
        sMaterialID mat = mesh->material;
        sShaderID shader = mat->shader;
        laMatrix inv_camera = laInverted(camera->user->transform.global);
        if (sShaderBind(shader) + sMaterialBind(mat))
        {
            sShaderUnbindLights(shader);
            for (size_t l=0; l<lights_count; l++) {
                glc(sShaderBindLight(shader, lights_list[l]));
            }
            glc(sShaderBindUniformIntToID(shader, shader->base_vars[lPointCount], shader->point_light_count));
            glc(sShaderBindUniformIntToID(shader, shader->base_vars[lSpotCount], shader->spotlight_count));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjection], camera->projection.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjectionInv], camera->projection_inv.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransform], inv_camera.a, 16));
        }
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vObjectTransform], obj->transform.global.a, 16));
        glc(sMeshDraw(draw_list[i]->visual_component));
    }
}

sCameraComponentID sCameraComponentCreateForwardRenderer(uint16_t width, uint16_t height, float FOV)
{
    sCameraComponentID camera = sCameraComponentCreate(width, height);
    camera->rpclbk = sForwardRenderCallback;
    camera->projection = laPerspective(
        width,
        height,
        400.0f, 0.02f, FOV
    );
    camera->projection_inv = laInverted(camera->projection);
    return camera;
}

extern sTextureID cubemap;
static void sDeferredRenderCallback(sCameraComponentID camera, sGameObjectID* draw_list, sGameObjectID* lights_list, sTextureID skybox)
{
    const char* gBufferComponents[] = {"gAlbedo", "gSpace", "gMasks", "gAmbient"};
    glc(sFrameBufferBind(&camera->framebuffer, gAlbedoBit | gSpaceBit | gMasksBit | gAmbientBit));
    glc(glEnable(GL_DEPTH_TEST));
    sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
    sMaterialBind(0);
    size_t objects_count = sListGetSize(draw_list);
    size_t shaders_count = sListGetSize(camera->shaders);
    laMatrix inv_camera = laInverted(camera->user->transform.global);
    
    for (size_t i=0; i<objects_count; i++)
    {
        if (!draw_list[i]->visual_component) continue;
        sGameObjectID obj = draw_list[i];
        sMeshID mesh = obj->visual_component;
        sMaterialID mat = mesh->material;
        sShaderID shader = mat->shader;
        (sShaderBind(shader) + sMaterialBind(mat));
        {
            glc(sShaderBindUniformIntToID(shader, shader->base_vars[lPointCount], shader->point_light_count));
            glc(sShaderBindUniformIntToID(shader, shader->base_vars[lSpotCount], shader->spotlight_count));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjection], camera->projection.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjectionInv], camera->projection_inv.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransform], inv_camera.a, 16));
        }
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vObjectTransform], obj->transform.global.a, 16));
        glc(sMeshDraw(draw_list[i]->visual_component));
    }
    glc(sShaderBind(0));
    if (shaders_count==1) {
        glc(sFrameBufferBind(0, 0));
    } else {
        glc(sFrameBufferBind(&camera->framebuffer, gOutputDiffuseBit | gOutputSpecularBit));
    }
    glc(glDisable(GL_DEPTH_TEST));
    sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
    glc(sShaderBind(camera->shaders[0]));
    glc(sShaderBindUniformFloat(camera->shaders[0], "iTime", SDL_GetTicks()/1000.0));
        
    for (int i=0; i<4; i++) {
        glc(sShaderBindTexture(camera->shaders[0], (char*)gBufferComponents[i], camera->framebuffer.color_render_targets[i]));
    }
    
    glc(sShaderBindTexture(camera->shaders[0], "gNoise", 0));
    glc(sShaderBindTexture(camera->shaders[0], "cubemap", 0));
    sShaderBindLights(camera->shaders[0], lights_list);
    glc(sShaderBindUniformFloat(camera->shaders[0], (char*)"width", (float)camera->framebuffer.width));
    glc(sShaderBindUniformFloat(camera->shaders[0], (char*)"height", (float)camera->framebuffer.height));
    glc(sShaderBindUniformFloatArray(camera->shaders[0], "vCameraProjectionInv", camera->projection_inv.a, 16));
    glc(sShaderBindUniformFloatArray(camera->shaders[0], "vCameraTransform", camera->user->transform.global.a, 16));
    glc(sShaderBindTexture(camera->shaders[0], "gNoise", blue_noise));
    glc(sShaderBindTexture(camera->shaders[0], "cubemap", skybox));
    glc(sMeshDraw(screen_plane));

    for (int shid=1; shid<shaders_count; shid++) {
        uint16_t gInputIndex = shid&1 ? gOutputAIndex : gOutputBIndex;
        uint16_t gOutputBit  = shid&1 ? gOutputBBit   : gOutputABit;
        sShaderID shader = camera->shaders[shid];
        glc(sShaderBind(shader));
        if (shid==shaders_count-1) {
            glc(sFrameBufferBind(0, 0));
        } else {
            glc(sFrameBufferBind(&camera->framebuffer, gOutputBit));
        }
        glc(glDisable(GL_DEPTH_TEST));
        sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
        for (int i=0; i<4; i++) {
            glc(sShaderBindTexture(shader, (char*)gBufferComponents[i], camera->framebuffer.color_render_targets[i]));
        }
        glc(sShaderBindUniformFloat(shader, "iTime", SDL_GetTicks()/1000.0));
        glc(sShaderBindTexture(shader, "gNoise", 0));
        glc(sShaderBindTexture(shader, "cubemap", 0));
        glc(sShaderBindTexture(shader, "gLDiffuse", 0));
        glc(sShaderBindTexture(shader, "gLSpecular", 0));
        glc(sShaderBindTexture(shader, "gOutput", 0));
        
        glc(sShaderBindTexture(shader, "gNoise", blue_noise));
        glc(sShaderBindTexture(shader, "cubemap", skybox));
        glc(sShaderBindTexture(shader, "gLDiffuse", camera->framebuffer.color_render_targets[gOutputDiffuseIndex]));
        glc(sShaderBindTexture(shader, "gLSpecular", camera->framebuffer.color_render_targets[gOutputSpecularIndex]));
        glc(sShaderBindTexture(shader, "gOutput", camera->framebuffer.color_render_targets[gInputIndex]));
        glc(sShaderBindUniformInt(shader, (char*)"gFilterPass", (int)shid));
        glc(sShaderBindUniformFloat(shader, (char*)"width", (float)camera->framebuffer.width));
        glc(sShaderBindUniformFloat(shader, (char*)"height", (float)camera->framebuffer.height));
        glc(sShaderBindUniformFloatArray(shader, (char*)"vCameraProjection", camera->projection.a, 16));
        glc(sShaderBindUniformFloatArray(shader, (char*)"vCameraProjectionInv", camera->projection_inv.a, 16));
        glc(sShaderBindUniformFloatArray(shader, (char*)"vCameraTransform", camera->user->transform.global.a, 16));
        glc(sShaderBindUniformFloatArray(shader, (char*)"vCameraTransformInv", laInverted(camera->user->transform.global).a, 16));
        
        glc(sMeshDraw(screen_plane));
    }
}

sCameraComponentID sCameraInitDeferredRenderer(uint16_t width, uint16_t height, float FOV)
{
    if (!screen_plane) screen_plane = sMeshCreateScreenPlane();
    sShaderID deferred_shader = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/deferred.glsl");
    sShaderID ssao_shader = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/ssao.glsl");
    sShaderID ssr_shader = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/SSR.glsl");
    sShaderID mcf = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/blur.glsl");
    sShaderID points_filter = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/median.glsl");
    sCameraComponentID camera = sCameraComponentCreate(width, height);
    sTextureID albedo  = sTextureCreate2D("gBufferAlbedo", width, height, RGBA8I, 0, 0, null);
    sTextureID space   = sTextureCreate2D("gBufferSpace", width, height, RGBA16I, 0, 0, null);
    sTextureID masks   = sTextureCreate2D("gBufferMasks", width, height, RGBA8I, 0, 0, null);
    sTextureID ambient = sTextureCreate2D("gBufferAmbient", width, height, RGB16F, 0, 0, null);

    sTextureID output_a = sTextureCreate2D("RenderOutA", width, height, RGB16F, 0, 0, null);
    sTextureID output_b = sTextureCreate2D("RenderOutB", width, height, RGB16F, 0, 0, null);
    sTextureID output_diffuse = sTextureCreate2D("RenderLightingDiffuse", width, height, RGB16F, 0, 0, null);
    sTextureID output_specular = sTextureCreate2D("RenderLightingSpecular", width, height, RGB16F, 0, 0, null);
    uint16_t nsize = 16;
    /*if (access("blue_noise.dds", F_OK)!=-1) {
        blue_noise = sTextureLoadDDS("blue_noise.dds");
    }
    else*/
    {
        blue_noise = sTextureGenerateBlueNoise(nsize*16, nsize);
    }
    /*if (access("blue_noise.png", F_OK)!=-1) {
        blue_noise = sTextureLoad("blue_noise.png", "BlueNoise");
    } else {
        blue_noise = sTextureGenerateBlueNoise(nsize*16, nsize);
        sTextureSave(blue_noise, "blue_noise.png");
    }*/
    //blue_noise = sTextureGenerateWhiteNoise(65468, 256, 256);
    //sTextureSave(blue_noise, "white_noise.png");
    sTextureSetTiling(blue_noise, sTextureRepeat);
    glc(glBindTexture(GL_TEXTURE_2D, blue_noise->ID));
	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR));
	glc(glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR));
	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_R, GL_REPEAT));
	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT));
	glc(glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT));
    sCameraAddFilter(camera, deferred_shader);
    sCameraAddFilter(camera, ssao_shader);
    sCameraAddFilter(camera, ssr_shader);
    sCameraAddFilter(camera, sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/predenoiser.glsl"));
    //sCameraAddFilter(camera, sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/denoisers/smart_denoise.glsl"));
    for (int i=0; i<0; i++) {
        sCameraAddFilter(camera, mcf);
    }
    sCameraAddFilter(camera, points_filter);
    //sCameraAddFilter(camera, sky_generator);
    camera->framebuffer = sFrameBufferCreate(width, height, 1);
    sFrameBufferAddRenderTarget(&camera->framebuffer, albedo);
    sFrameBufferAddRenderTarget(&camera->framebuffer, space);
    sFrameBufferAddRenderTarget(&camera->framebuffer, masks);
    sFrameBufferAddRenderTarget(&camera->framebuffer, ambient);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_a);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_b);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_diffuse);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_specular);

    camera->rpclbk = sDeferredRenderCallback;
    camera->projection = laPerspective(
        width,
        height,
        400.0f, 0.02f, FOV
    );
    camera->projection_inv = laInverted(camera->projection);
    return camera;
}

void sCameraComponentDelete(sCameraComponentID camera)
{
    if (!camera) return;
    sFrameBufferDelete(&camera->framebuffer);
    size_t fc = sListGetSize(camera->shaders);
    for (size_t i=0; i<fc; i++)
    {
        sListPopItem(camera->shaders[i]->render_users, camera);
    }
    camera->rpclbk = 0;
    camera->user = 0;
    sListPopItem(cameras, camera);
    sDelete(camera->shaders);
    sDelete(camera);
}

#ifdef __cplusplus
}
#endif
