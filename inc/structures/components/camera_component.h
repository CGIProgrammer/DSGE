/*
 * camera_component.h
 *
 *  Created on: 16 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef CAMERA_H
#define CAMERA_H

#include "structures/types.h"
#include "structures/framebuffer.h"
#include "structures/components/light_component.h"
#include "structures/shader.h"
#include "linalg.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup camera_component sCameraComponent
 *
 * Компонент камеры для sGameObject. Храненит буфера кадра
 * и фильтры постобработки.
 */

typedef struct sCameraComponent {
    laMatrix projection;
    laMatrix projection_inv;
    laMatrix dither;
    bool parallel;
    float field_of_view;
    sFrameBuffer framebuffer;
    sFrameBuffer pp_framebuffer;
    sShaderID* shaders;
    sShaderID txaa;
    sShaderID soap;
    bool tss;
    sCameraRenderPipelineCallback rpclbk;
    sGameObjectID user;
} sCameraComponent;


/**
 * @brief Создание компонента камеры
 * @param width Ширина кадра фреймбуфера
 * @param height Высота кадра фреймбуфера
 * @return Возвращает новый компонент камеры
 * @ingroup camera_component
 */
sCameraComponentID sCameraComponentCreate(uint16_t width, uint16_t height);


/**
 * @brief Удаление компонента камеры
 * @ingroup camera_component
 */
void sCameraComponentDelete(sCameraComponentID camera);


/**
 * @brief Добавление фильтра постобработки
 * @param camera Компонент камеры
 * @param shader Фильтр в виде шейдера
 * @ingroup camera_component
 */
void sCameraAddFilter(sCameraComponentID camera, sShaderID shader);


/**
 * @brief Отсоединение фильтра постобработки
 * @param camera Компонент камеры
 * @param shader Отсоединяемый фильтр
 * @ingroup camera_component
 */
void sCameraPopFilter(sCameraComponentID camera, sShaderID shader);


/**
 * @brief Отсоединение всех фильтров от камеры
 * @param camera Компонент камеры
 * @ingroup camera_component
 */
void sCameraClearFilters(sCameraComponentID camera);


/**
 * @brief Инициализация прямого рендера (forward rendering)
 * @param camera Компонент камеры
 * @param width Разрешение по горизонтали (ширина)
 * @param height Разрешение по вертикали (высота)
 * @param FOV Угол обзора по горизонтали
 * @ingroup camera_component
 */
sCameraComponentID sCameraComponentCreateForwardRenderer(uint16_t width, uint16_t height, float FOV);


/**
 * @brief Инициализация отложенного рендера (deferred rendering)
 * @param camera Компонент камеры
 * @ingroup camera_component
 */
sCameraComponentID sCameraInitDeferredRenderer(uint16_t width, uint16_t height, float FOV, bool tss);


/**
 * @brief Создание текстуры неба
 * @todo Желательно перенести эту функцию в более подобающее место
 * 
 */
void sCameraComponentBakeSkybox(sTextureID tex);

#ifdef __cplusplus
}
#endif

#endif
