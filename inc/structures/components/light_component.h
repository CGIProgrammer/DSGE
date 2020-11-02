/*
 * camera_component.h
 *
 *  Created on: 22 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef LIGHT_COMPONENT_H
#define LIGHT_COMPONENT_H

#include "structures/types.h"
#include "structures/game_object.h"
#include "linalg.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    sLightPoint = 1,
    sLightSpot,
    sLightParallel = 3,
    sLightSun = 3
} sLightType;


typedef struct sLightComponent {
    sLightType type;
    sColor color;
    laMatrix projection;
    laMatrix projection_inv;
    bool parallel;
    float field_of_view;
    float spot_smooth;
    float zfar;
    float znear;
    sFrameBuffer shadow_buffer;
    sLightRenderPipelineCallback rpclbk;
    sGameObjectID user;
} sLightComponent;


/**
 * @brief Создание буфера теней
 * @param size Разрешение буфера
 * @param lt Тип источника света
 * sLightPoint - точечный источник (светит равномерно по всем сторонам)
 * sLightSpot - ограниченный по углу направленный истоник (фонарик, прожектор)
 * sLightParallel - направленный бесконечно удалённый ис точник (солнце)
 * 
 * @ingroup light_component
 */
sLightComponentID sLightCreateShadowBuffer(uint16_t size, sLightType lt);


/**
 * @brief Удаление компонента источника света
 * @ingroup light_component
 */
void sLightComponentDelete(sLightComponentID camera);

#ifdef __cplusplus
}
#endif

#endif

