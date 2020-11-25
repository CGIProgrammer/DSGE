#include "structures/components/light_component.h"

#ifdef __cplusplus
extern "C" {
#endif

static sLightComponentID* lights;

sLightComponentID sLightComponentCreate(uint16_t width, uint16_t height)
{
    sLightComponentID light = sNew(sLightComponent);
    sListPushBack(lights, light);
    light->shadow_buffer = sFrameBufferCreate(width, height, 1);
    return light;
}

static void sLightRenderSingleShadowMap(sLightComponentID light, sGameObjectID* draw_list)
{
    size_t objects_count = sListGetSize(draw_list);
    sFrameBufferFillColor((sColor){0.0, 0.0, 0.0, 0.0});
    //glc(glClear(GL_DEPTH_BUFFER_BIT));

    for (size_t i=0; i<objects_count; i++)
    {
        if (!draw_list[i]->visual_component) continue;
        sGameObjectID obj = draw_list[i];
        sMeshID mesh = obj->visual_component;
        sMaterialID mat = mesh->material;
        sShaderID shader = mat->shader_shadow;
        laMatrix inv_light = laInverted(light->user->transform.global);
        if (sShaderBind(shader) + sMaterialBind(mat))
        {
            glc(sShaderBindUniformFloatToID(shader, shader->base_vars[fDistanceFormat], light->type==sLightSun));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjection], light->projection.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraProjectionInv], light->projection_inv.a, 16));
            glc(sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vCameraTransform], inv_light.a, 16));
        }
        sShaderBindUniformFloatArrayToID(shader, shader->base_vars[vObjectTransform], obj->transform.global.a, 16);
        sMeshDraw(draw_list[i]->visual_component);
    }
}

static void sLightShadowRenderCallback(sLightComponentID light, sGameObjectID* draw_list)
{
    sFrameBufferBind(&light->shadow_buffer, 1, 1);
    sLightRenderSingleShadowMap(light, draw_list);
}

static void sLightShadowCubeRenderCallback(sLightComponentID light, sGameObjectID* draw_list)
{
	sGameObjectID go = light->user;
	laMatrix sides[] = {
			go->transform.global,
			go->transform.global,
			go->transform.global,
			go->transform.global,
			go->transform.global,
			go->transform.global
		};
		laMatrixSetXDirection(sides, 0,-1, 0);
		laMatrixSetYDirection(sides, 0, 0, 1);
		laMatrixSetZDirection(sides,-1, 0, 0);

		laMatrixSetXDirection(sides+1, 0, 1, 0);
		laMatrixSetYDirection(sides+1, 0, 0, 1);
		laMatrixSetZDirection(sides+1, 1, 0, 0);

		laMatrixSetXDirection(sides+4, 1, 0, 0);
		laMatrixSetYDirection(sides+4, 0, 0, 1);
		laMatrixSetZDirection(sides+4, 0,-1, 0);

		laMatrixSetXDirection(sides+5,-1, 0, 0);
		laMatrixSetYDirection(sides+5, 0, 0, 1);
		laMatrixSetZDirection(sides+5, 0, 1, 0);

		laMatrixSetXDirection(sides+3, 1, 0, 0);
		laMatrixSetYDirection(sides+3, 0,-1, 0);
		laMatrixSetZDirection(sides+3, 0, 0,-1);

		laMatrixSetXDirection(sides+2, 1, 0, 0);
		laMatrixSetYDirection(sides+2, 0, 1, 0);
		laMatrixSetZDirection(sides+2, 0, 0, 1);

		sFrameBufferBind(0, 0, 0);
		for (int i=0; i<6; i++)
		{
			go->transform.global = sides[i];
			sFrameBufferBind(&light->shadow_buffer, 2<<i, 1);
			sLightRenderSingleShadowMap(light, draw_list);
		}
}

sLightComponentID sLightCreateShadowBuffer(uint16_t size, sLightType lt, bool shadow)
{
    sLightComponentID light = sLightComponentCreate(size, size);
    sTextureID target = 0;
    char name[256];
    
    light->spot_smooth = 0.9f;
    light->color = (sColor){0.0, 0.0, 0.0, 0.0};
    light->zfar = 300.0f;
    light->znear = 10.1f;
    light->type = lt;
    
    switch (lt)
    {
    case sLightPoint:
        if (shadow) {
            target = sTextureCreateCubemap(name, size, size, RED16F, 0, 0);
            sprintf(target->name, "sLightPointShadowBuffer (%hu x %hu, %p)", size, size, target);
            sTextureCubeSplit(target);
            sFrameBufferAddRenderTarget(&light->shadow_buffer, target);
            for (int i=0; i<6; i++)
            {
                sFrameBufferAddRenderTarget(&light->shadow_buffer, target->sides[i]);
            }
            light->rpclbk = sLightShadowCubeRenderCallback;
        }
        light->projection = laPerspective(size, size, light->zfar, light->znear, 90.0);
        break;
    case sLightSpot:
        if (shadow) {
            target = sTextureCreate2D(name, size, size, RED16F, 0, 0, null);
            sprintf(target->name, "sLightSpotShadowBuffer (%hu x %hu, %p)", size, size, target);
            sFrameBufferAddRenderTarget(&light->shadow_buffer, target);
            light->rpclbk = sLightShadowRenderCallback;
        }
        light->field_of_view = 5.0f;
        light->zfar = 120.0f;
        light->projection = laPerspective(size, size, light->zfar, light->znear, light->field_of_view);
        break;
    case sLightParallel:
        if (shadow) {
            target = sTextureCreate2D(name, size, size, RED16F, 0, 0, null);
            sprintf(target->name, "sLightParallelShadowBuffer (%hu x %hu, %p)", size, size, target);
            light->rpclbk = sLightShadowRenderCallback;
        }
        light->field_of_view = 40.0f;
        light->projection = laOrtho(size, size, light->field_of_view, light->zfar, light->znear);
        break;
    default:
        fprintf(stderr, "Wrong light type\n");
        exit(-1);
    }
    if (shadow) {
        sFrameBufferAddRenderTarget(&light->shadow_buffer, target);
    }
    return light;
}

void sLightComponentDelete(sLightComponentID light)
{
    if (!light) return;
    sFrameBufferDelete(&light->shadow_buffer);
    light->rpclbk = 0;
    light->user = 0;
    sListPopItem(lights, light);
    sDelete(light);
}
#ifdef __cplusplus
}
#endif
