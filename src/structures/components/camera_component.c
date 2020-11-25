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
    .color_render_targets = {0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0},
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
    
    sFrameBufferBind(&skybox_fb, 0b111111, 0);

    sShaderBind(sky_generator);
    glc(sMeshDraw(screen_plane));
}

sCameraComponentID sCameraComponentCreate(uint16_t width, uint16_t height)
{
    sCameraComponentID camera = sNew(sCameraComponent);
    camera->framebuffer.width = width;
    camera->framebuffer.height = height;
    camera->dither = laIdentity;
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
    glc(sFrameBufferBind(&camera->framebuffer, 0, 1));
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
            glc(sShaderBindTexture(shader, "cubemap", skybox));
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
    sFrameBufferSetStd(width, height);
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
static int iteration = 0;
static float dither[][2] = {
    //{-1,-1},
    //{ 1,-1},
    //{-1, 1},
    //{ 1, 1},
    //{-1,-1},
    //{ 1,-1},
    //{-1, 1},
    //{ 1, 1},
    //{-1,-1},
    //{ 1,-1},
    //{-1, 1},
    //{ 1, 1},
    //{-1,-1},
    //{ 1,-1},
    //{-1, 1},
    //{ 1, 1},
    
    {-0.98732421,  0.859431  },
    {-0.65821248, -0.37457928},
    { 0.25716888,  0.1231111 },
    { 0.43520789, -0.58960568},
    { 0.18049204,  0.1479625 },
    { 0.47397861,  0.66341217},
    { 0.10755945, -0.68278827},
    { 0.2788744 , -0.62489427},
    {-0.71277244,  0.25320682},
    {-0.3370736 ,  0.28901948},
    { 0.36766457, -0.10139287},
    { 0.29690737, -0.79951376},
    {-0.68917806,  0.4233432 },
    { 0.15096014,  0.85329404},
    {-0.20626524, -0.60187284},
    { 0.34081387, -0.86511024}
};

static int object_sort_by_mesh(const void * val1, const void * val2)
{
    sGameObjectID obj1 = *((sGameObjectID*)val1);
    sGameObjectID obj2 = *((sGameObjectID*)val2);
    return (intptr_t)obj1->visual_component - (intptr_t)obj2->visual_component;
}

static void sDeferredRenderCallback(sCameraComponentID camera, sGameObjectID* draw_list, sGameObjectID* lights_list, sTextureID skybox)
{
    const char* gBufferComponents[] = {"gAlbedo", "gSpace", "gMasks", "gAmbient", "gVectors"};
    glc(sFrameBufferBind(&camera->framebuffer, gAlbedoBit | gSpaceBit | gMasksBit | gAmbientBit | gVectorsBit, 1));
    glc(glEnable(GL_DEPTH_TEST));
    sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
    sMaterialBind(0);
    laMatrix pdither = camera->dither;
    float aspect = (float)camera->framebuffer.height / camera->framebuffer.width;
    camera->dither = laRotationXYZ(
        (dither[iteration][1]*0.5)*radians(camera->field_of_view)/camera->framebuffer.width,
        (dither[iteration][0]*0.5)*radians(camera->field_of_view)*aspect/camera->framebuffer.height,
        0
    );
    qsort(draw_list, sListGetSize(draw_list), sizeof(sGameObjectID), object_sort_by_mesh);
    size_t objects_count = sListGetSize(draw_list);
    size_t shaders_count = sListGetSize(camera->shaders);
    laMatrix ori_camera = laMul(camera->user->transform.global, camera->dither);
    laMatrix inv_camera = laInverted(ori_camera);
    laMatrix inv_camera_stable = laInverted(camera->user->transform.global);
    laMatrix ori_camera_prev = laMul(camera->user->transform.global_prev_1, camera->dither);
    laMatrix inv_camera_prev = laInverted(ori_camera_prev);
    //iteration = 10;
    for (size_t i=0; i<objects_count; i++)
    {
        if (!draw_list[i]->visual_component) continue;
        sGameObjectID obj = draw_list[i];
        sMeshID mesh = obj->visual_component;
        sMaterialID mat = mesh->material;
        sShaderID shader = mat->shader;
        sShaderBind(shader);
        sMaterialBind(mat);
        {
            glc(sShaderBindUniformIntToID(shader, shader->base_vars[lPointCount], shader->point_light_count));
            glc(sShaderBindUniformIntToID(shader, shader->base_vars[lSpotCount], shader->spotlight_count));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjection], camera->projection.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjectionInv], camera->projection_inv.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransformStable], inv_camera_stable.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransform], inv_camera.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransformPrev], inv_camera_prev.a, 16));
        }
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vObjectTransform], obj->transform.global.a, 16));
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vObjectTransformPrev], obj->transform.global_prev_1.a, 16));
        glc(glEnable(GL_CULL_FACE));
        glc(glCullFace(GL_BACK));
        glc(sMeshDraw(draw_list[i]->visual_component));

        obj->transform.global_prev_2 = obj->transform.global_prev_1;
        obj->transform.global_prev_1 = obj->transform.global;
    }

    glc(glDisable(GL_CULL_FACE));
    
    glc(sShaderBind(0));
    if (shaders_count==1) {
        glc(sFrameBufferBind(0, 0, 0));
    } else {
        glc(sFrameBufferBind(&camera->framebuffer, gOutputDiffuseBit | gOutputSpecularBit, 0));
    }
    glc(glDisable(GL_DEPTH_TEST));
    glc(glDisable(GL_ALPHA_TEST));
    sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
    glc(sShaderBind(camera->shaders[0]));
    glc(sShaderBindUniformFloat(camera->shaders[0], "iTime", SDL_GetTicks()/1000.0));
        
    for (size_t i=0; i<sizeof(gBufferComponents)/sizeof(gBufferComponents[0]); i++) {
        glc(sShaderBindTexture(camera->shaders[0], (char*)gBufferComponents[i], camera->framebuffer.color_render_targets[i]));
    }
    
    glc(sShaderBindTexture(camera->shaders[0], "gNoise", 0));
    glc(sShaderBindTexture(camera->shaders[0], "cubemap", 0));
    sShaderBindLights(camera->shaders[0], lights_list);
    glc(sShaderBindUniformFloatArrayToID(camera->shaders[0], camera->shaders[0]->base_vars[gResolution], (float[]){camera->framebuffer.width, camera->framebuffer.height}, 2));
    glc(sShaderBindUniformFloatArray(camera->shaders[0], "vCameraProjectionInv", camera->projection_inv.a, 16));
    glc(sShaderBindUniformFloatArray(camera->shaders[0], "vCameraTransform", ori_camera.a, 16));
    glc(sShaderBindTexture(camera->shaders[0], "gNoise", blue_noise));
    glc(sShaderBindTexture(camera->shaders[0], "cubemap", skybox));
    glc(sShaderBindUniformInt(camera->shaders[0], (char*)"gDitherIteration", iteration));
    glc(sMeshDraw(screen_plane));
    uint16_t gInputIndex, gOutputBit;
    uint16_t gOutputIndex = gOutputAIndex;
    uint16_t gAccIn  = iteration&1 ? gRenderAccumulator1 : gRenderAccumulator2;
    uint16_t gAccOut = iteration&1 ? gRenderAccumulator2 : gRenderAccumulator1;
    uint16_t gAccTarg = iteration&1 ? gRenderAccumulator2Bit : gRenderAccumulator1Bit;
    uint16_t gAccTarg2 = iteration&1 ? gRenderAccumulator1Bit : gRenderAccumulator2Bit;

    sFrameBufferID fb = camera->tss ? &camera->pp_framebuffer : &camera->framebuffer;
    sTextureID acc = fb->color_render_targets[gAccOut];
    sTextureID acc2 = fb->color_render_targets[gAccIn];
    for (size_t shid=1; shid<shaders_count; shid++) {
        gInputIndex  = shid&1 ? gOutputAIndex : gOutputBIndex;
        gOutputIndex = shid&1 ? gOutputBIndex : gOutputAIndex;
        gOutputBit   = shid&1 ? gOutputBBit   : gOutputABit;
        
        sShaderID shader = camera->shaders[shid];
        glc(sShaderBind(shader));
        //if (shid==shaders_count-1) {
        //    glc(sFrameBufferBind(0, 0));
        //} else {
            glc(sFrameBufferBind(&camera->framebuffer, gOutputBit, 0));
        //}
        glc(glDisable(GL_DEPTH_TEST));
        glc(glDisable(GL_ALPHA_TEST));
        sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
        for (int i=0; i<sizeof(gBufferComponents)/sizeof(gBufferComponents[0]); i++) {
            glc(sShaderBindTexture(shader, (char*)gBufferComponents[i], camera->framebuffer.color_render_targets[i]));
        }
        glc(sShaderBindUniformFloat(shader, (char*)"iTime", SDL_GetTicks()/1000.0));
        glc(sShaderBindTexture(shader, (char*)"gNoise", 0));
        glc(sShaderBindTexture(shader, (char*)"cubemap", 0));
        glc(sShaderBindTexture(shader, (char*)"gLDiffuse", 0));
        glc(sShaderBindTexture(shader, (char*)"gLSpecular", 0));
        glc(sShaderBindTexture(shader, (char*)"gOutput", 0));
        
        glc(sShaderBindTexture(shader, (char*)"gNoise", blue_noise));
        glc(sShaderBindTexture(shader, (char*)"cubemap", skybox));
        glc(sShaderBindTexture(shader, (char*)"gLDiffuse", camera->framebuffer.color_render_targets[gOutputDiffuseIndex]));
        glc(sShaderBindTexture(shader, (char*)"gLSpecular", camera->framebuffer.color_render_targets[gOutputSpecularIndex]));
        glc(sShaderBindTexture(shader, (char*)"gOutput", camera->framebuffer.color_render_targets[gInputIndex]));
        glc(sShaderBindTexture(shader, (char*)"gAccumulator", camera->framebuffer.color_render_targets[gAccIn]));
        glc(sShaderBindTexture(shader, (char*)"gVectors", camera->framebuffer.color_render_targets[gVectors]));
        glc(sShaderBindUniformInt(shader, (char*)"gDitherIteration", iteration));
        glc(sShaderBindUniformInt(shader, (char*)"gFilterPass", (int)shid));
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[gResolution], (float[]){camera->framebuffer.width, camera->framebuffer.height}, 2));
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjection], camera->projection.a, 16));
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjectionInv], camera->projection_inv.a, 16));
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransform], ori_camera.a, 16));
        glc(sShaderBindUniformFloatArray(shader, (char*)"vCameraTransformInv", inv_camera.a, 16));
        glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransformPrev], ori_camera_prev.a, 16));
        
        glc(sMeshDraw(screen_plane));
    }
    glc(sFrameBufferBind(fb, gAccTarg, 0));
    //glViewport(0, 0, camera->framebuffer.width*2, camera->framebuffer.height*2);
    glc(sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0}));
    glc(sShaderBind(camera->txaa));
    glc(sShaderBindUniformInt(camera->txaa, (char*)"gDitherIteration", iteration));
    glc(sShaderBindUniformFloatArray(camera->txaa, (char*)"gDither", dither[iteration], 2));
    for (int i=0; i<sizeof(gBufferComponents)/sizeof(gBufferComponents[0]); i++) {
        glc(sShaderBindTexture(camera->txaa, (char*)gBufferComponents[i], fb->color_render_targets[i]));
    }
    glc(sShaderBindTexture(camera->txaa, (char*)"gOutput", fb->color_render_targets[gOutputIndex]));
    glc(sShaderBindTexture(camera->txaa, (char*)"gAccumulator", fb->color_render_targets[gAccIn]));
    glc(sShaderBindTexture(camera->txaa, (char*)"gVectors", fb->color_render_targets[gVectors]));
    glc(sShaderBindUniformFloatArrayToID(camera->txaa, camera->txaa->base_vars[gResolution], (float[]){fb->width, fb->height}, 2));
    glc(sMeshDraw(screen_plane));

    if (iteration%8==0) {
        glc(sFrameBufferBind(fb, gAccTarg2, 0));
        glc(sShaderBind(camera->soap));
        glc(sShaderBindUniformFloatArrayToID(camera->soap, camera->soap->base_vars[gResolution], (float[]){fb->width, fb->height}, 2));
        glc(sShaderBindTexture(camera->soap, (char*)"gOutput", acc));
        for (int i=0; i<sizeof(gBufferComponents)/sizeof(gBufferComponents[0]); i++) {
            glc(sShaderBindTexture(camera->soap, (char*)gBufferComponents[i], fb->color_render_targets[i]));
        }
        glc(sMeshDraw(screen_plane));
        acc = fb->color_render_targets[gRenderAccumulator1];
        fb->color_render_targets[gRenderAccumulator1] = fb->color_render_targets[gRenderAccumulator2];
        fb->color_render_targets[gRenderAccumulator2] = acc;

        glc(sFrameBufferBind(0, 0, 0));
        sTextureDraw(acc2, 0, 0);
    } else {
        glc(sFrameBufferBind(0, 0, 0));
        sTextureDraw(acc, 0, 0);
    }
        
    /*glc(sFrameBufferBind(&camera->framebuffer, gRenderAccumulator1Bit, 0));
    glDisable(GL_DEPTH_TEST);
    sTextureDraw(camera->framebuffer.color_render_targets[gOutputIndex], 0, 0);
    glc(sFrameBufferBind(0, 0, 0));
    glDisable(GL_DEPTH_TEST);
    sTextureDraw(camera->framebuffer.color_render_targets[gOutputIndex], 0, 0);*/

    camera->user->transform.global_prev_2 = camera->user->transform.global_prev_1;
    camera->user->transform.global_prev_1 = camera->user->transform.global;
    iteration = (iteration + 1)&15;

}

sCameraComponentID sCameraInitDeferredRenderer(uint16_t width, uint16_t height, float FOV, bool tss)
{
    if (!screen_plane) screen_plane = sMeshCreateScreenPlane();
    sFrameBufferSetStd(width, height);
    sShaderID deferred_shader = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/deferred.glsl");
    sShaderID ssao_shader = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/ssao.glsl");
    sShaderID ssr_shader = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/SSR.glsl");
    sShaderID mcf = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/blur.glsl");
    sShaderID points_filter = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/median.glsl");
    sCameraComponentID camera = sCameraComponentCreate(width, height);

    uint16_t hwidth  = tss ? width/2  : width;
    uint16_t hheight = tss ? height/2 : height;

    sTextureID albedo  = sTextureCreate2D("gBufferAlbedo",  hwidth, hheight, RGBA8I, 0, 0, null);
    sTextureID space   = sTextureCreate2D("gBufferSpace",   hwidth, hheight, RGBA16I, 1, 0, null);
    sTextureID masks   = sTextureCreate2D("gBufferMasks",   hwidth, hheight, RGBA8I, 0, 0, null);
    sTextureID ambient = sTextureCreate2D("gBufferAmbient", hwidth, hheight, RGB16F, 0, 0, null);
    sTextureID vector_map  = sTextureCreate2D("gVectors",   hwidth, hheight, RGBA16F, tss, 0, null);
    sTextureID output_a = sTextureCreate2D("gRenderOutA",   hwidth, hheight, RGBA16F, 0, 0, null);
    sTextureID output_b = sTextureCreate2D("gRenderOutB",   hwidth, hheight, RGBA16F, 0, 0, null);
    sTextureID output_diffuse  = sTextureCreate2D("gRenderLightingDiffuse",  hwidth, hheight, RGB16F, 0, 0, null);
    sTextureID output_specular = sTextureCreate2D("gRenderLightingSpecular", hwidth, hheight, RGB16F, 0, 0, null);

    sTextureID accumulator1 = sTextureCreate2D("gAccumulatorA", width, height, RGBA16F,  1, 0, null);
    sTextureID accumulator2 = sTextureCreate2D("gAccumulatorB", width, height, RGBA16F,  1, 0, null);

    camera->tss = tss;
    uint16_t nsize = 64;
    /*if (access("blue_noise.dds", F_OK)!=-1) {
        blue_noise = sTextureLoadDDS("blue_noise.dds");
    }
    else
    {
        blue_noise = sTextureGenerateBlueNoise(nsize*64, nsize);
    }*/
    if (access("blue_noise.png", F_OK)!=-1) {
        blue_noise = sTextureLoad("blue_noise.png", "BlueNoise");
    } else {
        blue_noise = sTextureGenerateBlueNoise(nsize*64, nsize);
        sTextureSave(blue_noise, "blue_noise.png");
    }
    //blue_noise = sTextureGenerateWhiteNoise(65468, 256, 256);
    //sTextureSave(blue_noise, "white_noise.png");
    sTextureSetTiling(blue_noise, sTextureRepeat);
    sCameraAddFilter(camera, deferred_shader);
    sCameraAddFilter(camera, ssao_shader);
    sCameraAddFilter(camera, sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/denoisers/old_denoiser.glsl"));
    /*for (int i=0; i<8; i++)
    {
        sCameraAddFilter(camera, mcf);
    }*/
    sCameraAddFilter(camera, ssr_shader);
    sCameraAddFilter(camera, points_filter);
    if (tss) {
        camera->txaa = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/txaa_ss.glsl");
    } else {
        camera->txaa = sShaderMakeFromFiles("data/shaders/screen_plane.glsl", "data/shaders/txaa.glsl");
    }
    camera->soap = mcf;
    //sCameraAddFilter(camera, mcf);
    camera->framebuffer = sFrameBufferCreate(hwidth, hheight, 1);
    sFrameBufferAddRenderTarget(&camera->framebuffer, albedo);
    sFrameBufferAddRenderTarget(&camera->framebuffer, space);
    sFrameBufferAddRenderTarget(&camera->framebuffer, masks);
    sFrameBufferAddRenderTarget(&camera->framebuffer, ambient);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_a);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_b);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_diffuse);
    sFrameBufferAddRenderTarget(&camera->framebuffer, output_specular);
    sFrameBufferAddRenderTarget(&camera->framebuffer, accumulator1);
    sFrameBufferAddRenderTarget(&camera->framebuffer, accumulator2);
    sFrameBufferAddRenderTarget(&camera->framebuffer, vector_map);
    if (tss) {
        camera->pp_framebuffer = sFrameBufferCreate(width, height, 0);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, albedo);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, space);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, masks);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, ambient);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, output_a);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, output_b);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, output_diffuse);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, output_specular);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, accumulator1);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, accumulator2);
        sFrameBufferAddRenderTarget(&camera->pp_framebuffer, vector_map);
    }

    camera->rpclbk = sDeferredRenderCallback;
    camera->projection = laPerspective(
        width,
        height,
        100.0f, 0.1f, FOV
    );
    camera->projection_inv = laInverted(camera->projection);
    camera->field_of_view = FOV;
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
