/*
 * scene.h
 *
 *  Created on: 09 июль. 2020 г.
 *      Author: ivan
 */


#ifndef SCENE_H
#define SCENE_H

#include "structures/types.h"
#include "structures/game_object.h"
#include "structures/texture.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup scene sScene
 * 
 * Сцена.
 * @todo Реализовать такт игры
 */

//typedef struct _sGameObject* sGameObjectID;
//typedef struct _sGameObject sGameObject;

typedef struct sScene
{
    char name[256];
    void (**behaviour)(struct sScene*);
    struct sGameObject* camera;
    struct sGameObject** objects;
    struct sGameObject** lights;
    struct sTexture* skybox;
    void* world;
} sScene;


/**
 * @brief Создание новоый сцены
 * @param name Имя
 * @return Возвращает указатель на новую сцену sSceneID
 * @ingroup scene
 */
sSceneID sSceneCreate(char* name);


/**
 * @brief Удаление сцены
 * @param scene Удаляемая сцена
 * @ingroup scene
 */
void sSceneDelete(sScene* scene);


/**
 * @brief Добавление объекта на сцену
 * @param object Добавляемый объект
 * @ingroup scene
 */
void sSceneAddObject(sSceneID, struct sGameObject* object);


/**
 * @brief Присоединение скайбокса к сцене
 * @param texture Присоединяемый скайбокс (должен иметь тип cubemap)
 * @ingroup scene
 */
void sSceneSetSkybox(sSceneID, struct sTexture* texture);


/**
 * @brief Установка камеры сцены
 * @param camera Камера, устанавливаемая в качестве вьюпорта
 * @ingroup scene
 */
void sSceneSetActiveCamera(sSceneID, struct sGameObject* camera);


/**
 * @brief Добавление функции, вызывающейся каждый такт игры
 * @param scene Сцена, к которой добавляется функция
 * @param callback Добавляемая функция.
 * @ingroup scene
 */
void sSceneAddBehaviourCallback(sSceneID scene, void (*callback)(sSceneID));


/**
 * @brief Удаление завершённых объектов
 * @ingroup scene
 */
void sSceneClearEndedObjects(sScene* scene);


/**
 * @brief Выполнение такта игры
 * @ingroup scene
 */
void sSceneWorldStep(sSceneID scene, float delta);


/**
 * @brief Визуализация сцены
 * @ingroup scene
 */
void sSceneDraw(sSceneID scene);


/**
 * @brief Загрузка сцены из текстового файла
 * @ingroup scene
 */
sSceneID sSceneLoadFromText(char* fname);


/**
 * @brief Загрузка сцены из бинарного файла
 * @ingroup scene
 */
sSceneID sSceneLoadBin(const char* fname);

/**
 * @brief Создание простой демосцены
 * @ingroup scene
 */
sSceneID sSceneMakeDemo(void);

#ifdef __cplusplus
}
#endif

#endif
