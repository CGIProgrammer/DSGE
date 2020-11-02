/*
 * game_object.h
 *
 *  Created on: 09 июль. 2020 г.
 *      Author: Ivan G
 */

#ifndef GAME_OBJECT_H
#define GAME_OBJECT_H

#include "structures/types.h"
#include "structures/mesh.h"
#include "structures/scene.h"
#include "linalg.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup game_object sGameObject
 * 
 * Объект сцены.
 * 
 * Представляет собой древовидную структуру с компонентами:
 * @li Трансформация (sObjectTransform)
 * @li Визуал (sMesh)
 * @li Твёрдое тело
 * @li Источник света
 * @li Камера
 * @li Скелет
 * 
 * @warning Созданные объекты контролируются сборщиком мусора,
 * поэтому, если объект не будет размещён на сцене или ему
 * не будет установлен фиктивный пользователь, объект удалится
 * на следующем такте игры.
 * 
 * @todo Реализовать компонент "камера"
 * @todo Реализовать компонент "источник света"
 * @todo Реализовать компонент "скелет"
 * @todo Реализовать компонент "физика"
 */

typedef struct sObjectTransform
{
    laMatrix local;
    laMatrix global;
    laMatrix global_prev_1;
    laMatrix global_prev_2;
} sObjectTransform;

typedef struct sGameObject
{
    char name[256];
    bool ended;
    bool hidden;
    uint32_t sleep_timer;
    sSceneID scene;
    sGameObjectID  parent;
    sGameObjectID* children;
    sObjectTransform transform;
    sMeshID visual_component;
    void*   physics_component;
    sLightComponentID   light_component;
    sCameraComponentID camera_component;
    void*   skeleton_component;
    sGameObjectCallback* behaviour;
	bool fake_user;
} sGameObject;


/**
 * @brief Создание нового объекта
 * @param name Имя
 * @return Возвращает указатель на новый sGameObject
 * @ingroup game_object
 */
sGameObjectID sGameObjectCreate(char* name);


/**
 * @brief Присоединение полисетки к объекту
 * @param obj Объект, которому присоединяется полисетка
 * @param visual Присоединяемая полисетка
 * @ingroup game_object
 */
void sGameObjectSetVisual(sGameObjectID obj, sMeshID visual);


/**
 * @brief Загрузка объекта из файла
 * @param fp Указатель файловый поток
 * @return Возвращает указатель на новый sGameObject
 * @todo Написать реализацию
 * @ingroup game_object
 * 
 */
sGameObjectID sGameObjectLoad(FILE* fp);


/**
 * @brief Добавление потомка объекту.
 * Контролирует соблюжение ацикличности
 * 
 * @param obj Указатель на объект, которому будет добавлен потомок
 * @param child Указатель на объект-потомок
 * @ingroup game_object
 * 
 */
void sGameObjectAddChild(sGameObjectID obj, sGameObjectID child);

/**
 * @brief Присоединение родителя объекту.
 * Если образуется цикл, то ничего не проиходит
 * Если родитель уже есть, то он отсоединяется.
 * 
 * @param obj Указатель на объект, к которому будет присоединён родитель
 * @param parent Указатель на объект-потомок.
 * @ingroup game_object
 * 
 */
void sGameObjectSetParent(sGameObjectID obj, sGameObjectID parent);


/**
 * @brief Добавление функции, вызывающейся каждый такт игры
 * @param obj Указатель на объект, к которому будет добавлена функция
 * @param callback Процедура, принимающая sGameObject в качестве аргумента.
 * @ingroup game_object
 * 
 */
void sObjectAddCallback(sGameObjectID obj, sGameObjectCallback callback);


/**
 * @brief Отсоединение родителя от объекта.
 * @ingroup game_object
 */
void sGameObjectRemoveParent(sGameObjectID);


/**
 * @brief Сделать объект невидимым
 * @ingroup game_object
 */
void sGameObjectHide(sGameObjectID);


/**
 * @brief Сделать объект видимым
 * @ingroup game_object
 */
void sGameObjectShow(sGameObjectID);


/**
 * @brief Установить видимость объекту
 * @ingroup game_object
 */
void sGameObjectSetVisibility(sGameObjectID, bool);


/**
 * @brief Применить трансформации потомков.
 * Рекурсивно выстраивает потомков относительно родителя в соответствии
 * с локальной перемещениями и поворотами.
 * Необходимо выполнять при перемещении или повороте объекта, у
 * которого есть потомки.
 * 
 * @ingroup game_object
 */
void sGameObjectApplyChildrenTransform(sGameObjectID);


/**
 * @brief Завершить объект.
 * Помечает объект для удаления со сцены в конце такта игры.
 * 
 * @ingroup game_object
 */
void sGameObjectEnd(sGameObjectID obj);


/**
 * @brief Уничтожить объект
 * Применяется исключительно вне callback-функций сцены и объектов.
 * 
 * @ingroup game_object
 */
void sGameObjectDelete(sGameObjectID);


void sGameObjectClear(void);

#ifdef __cplusplus
}
#endif

#endif
