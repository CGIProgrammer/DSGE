
/*
 * mesh.h
 *
 *  Created on: 9 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef MESH_H
#define MESH_H

#include <stdint.h>
#include "linalg.h"
#include "structures/types.h"
#include "structures/material.h"
#include "structures/texture.h"
#include "structures/game_object.h"
#include "structures/list.h"

/**
 * @defgroup mesh sMesh
 * 
 * Полигональная сетка.
 */

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sMesh
{
	char name[256];
	laMatrix* link_matrix;
	sVertex* vertices;
	index_t* indices;
	uint32_t ind_count;
	uint32_t vert_count;
	sMaterialID material;
	uint32_t VBO;
	uint32_t IBO;
	laMatrix bounding_box_size;
	laMatrix bounding_box_start;
	laMatrix bounding_box_end;
	uint16_t bones_indices[128];
	bool deformed;
	uint8_t uv_count;
    bool fake_user;
	sGameObjectID* users;
} sMesh;


/**
 * @brief Создание новой полисетки
 * @return Возвращает указатель на новую пустую полисетку
 * @ingroup mesh
 */
sMeshID sMeshCreate(char* name);


/**
 * @brief Создание плоскости для полноэкранного вывода текстур
 * 
 * @return Возвращаяет указатель на новую плоскость
 * @ingroup mesh
 */
sMeshID sMeshCreateScreenPlane(void);


/**
 * @brief Создание вершинного и индексного буферов
 * 
 * @param mesh Указатель на полисетку
 * @ingroup mesh
 */
void sMeshMakeDynamicBuffers(sMeshID mesh);


/**
 * @brief Обновление вершинного и индексного буферов
 * Предназначено для выполнения после изменения массива вершин
 * или массива индексов
 * 
 * @param mesh Указатель на полисетку
 * @param vertices Обновить массив вершин
 * @param indices Обновить массив индексов
 * @ingroup mesh
 */
void sMeshUpdateDynamicBuffers(sMeshID mesh, bool vertices, bool indices);


/**
 * @brief Присоединение материала к полисетке
 * @param mesh Указатель на полисетку
 * @param material Указатель на присоединяемый материал
 * @ingroup mesh
 */
void sMeshSetMaterial(sMeshID mesh, sMaterialID material);


/**
 * @brief Привязка буферов полисетки к активному шейдеру
 * @param mesh Указатель на полисетку
 * @ingroup mesh
 */
void sMeshBindBuffers(sMeshID mesh);


/**
 * @brief Удаление буферов полисетки
 * @param mesh Указатель на полисетку
 * @ingroup mesh
 */
void sMeshDeleteBuffers(sMeshID mesh);


/**
 * @brief Загрузка полисетки из файла
 * @param name Путь к файлу (абсолютный или относительный)
 * @return Возвращает указатель на полисетку с загруженными данными
 * @ingroup mesh
 */
sMeshID sMeshLoad(char* name);


/**
 * @brief Отрисовка полисетки
 * @param mesh Указатель на полисетку
 * @ingroup mesh
 */
void sMeshDraw(sMeshID mesh);


/**
 * @brief Добавление объекта в список пользователей 
 * @ingroup mesh
 */
void sMeshAddUser(sMeshID mesh, sGameObjectID object);


/**
 * @brief Удаление объекта их списка пользователей 
 * @ingroup mesh
 */
void sMeshRemoveUser(sMeshID mesh, sGameObjectID object);


/**
 * @brief Очистка списка пользователей полисетки
 * @ingroup mesh
 */
void sMeshRemoveUsers(sMeshID mesh);


/**
 * @brief Очистка списка пользователей полисетки
 * @ingroup mesh
 */
void sMeshRemoveUsers(sMeshID mesh);


/**
 * @brief Удаление полисетки
 * @param mesh Указатель на полисетку
 * @ingroup mesh
 */
void sMeshDelete(sMeshID mesh);


/**
 * @brief Удаление неиспользуемых полисеток
 * @ingroup mesh
 */
void sMeshClear(void);

#ifdef __cplusplus
}
#endif

#endif
