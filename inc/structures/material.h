
/*
 * material.h
 *
 *  Created on: 9 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef MATERIAL_H
#define MATERIAL_H

#include "structures/types.h"
#include "structures/shader.h"
#include "structures/texture.h"
#include "structures/mesh.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup material sMaterial
 * 
 * Материал полисетки. Модель поверхности, опрежеляющая её внешний вид.
 */

typedef struct sMaterial
{
	char name[256];
	float height_scale;
	sColor diffuse;
	sColor specular;
	float roughness;
	float metallic;
	float fresnel;
	sTextureID diffuse_texture;
	sTextureID specular_texture;
	sTextureID roughness_texture;
	sTextureID metallic_texture;
	sTextureID height_texture;
	sTextureID lightmap_texture;
	float tdx;
	float tdy;
	float glow;
	sShaderID shader;
	sShaderID shader_skeleton;
	sShaderID shader_shadow;
	sShaderID shader_skeleton_shadow;
    bool fake_user;
    sMeshID* mesh_users;
} sMaterial;

/**
 * @brief Создание пустого материала
 * @param name Имя
 * @return Возвращает указатель на новый sMaterial
 * @ingroup material
 */
sMaterialID sMaterialCreate(char* name);


/**
 * @brief Создание материала с шейдером по умолчанию
 * @param name Имя
 * @return Возвращает указатель на новый sMaterial
 * @ingroup material
 */
sMaterialID sMaterialCreateWithDefaultShader(char* name);


/**
 * @brief Удаление материала
 * @ingroup material
 */
void sMaterialDelete(sMaterialID);


/**
 * @brief Отсоединение текстуры от материала
 * @param material Материал от которого отсоединяется текстура
 * @param texture Указатель на отсоединяемую текстуру
 * @ingroup material
 */
void sMaterialDetachTexture(sMaterialID material, sTextureID texture);


/**
 * @brief Присоединение шейдера
 * Если в качестве шейдера передаётся null, шедйдер отсоединяется
 * 
 * @param material Материал, к которому присоединяется шейдер
 * @param shader Присоединяемый шейдер
 * @ingroup material
 */
void sMaterialSetShader(sMaterialID material, sShaderID shader, sShaderType type);


/**
 * @brief Присоединение диффузной текстуры (альбедо)
 * @ingroup material
 */
void sMaterialSetDiffuseMap(sMaterialID, sTextureID);


/**
 * @brief Присоединение бликовой карты
 * @ingroup material
 */
void sMaterialSetSpecularMap(sMaterialID, sTextureID);

/**
 * @brief Присоединение карты матовости
 * @ingroup material
 */
void sMaterialSetRoughnessMap(sMaterialID, sTextureID);


/**
 * @brief Присоединение карты зеркальных отражений
 * @ingroup material
 */
void sMaterialSetMetallicMap(sMaterialID, sTextureID);


/**
 * @brief Присоединение карты рельефа
 * @ingroup material
 */
void sMaterialSetHeightMap(sMaterialID, sTextureID);


/**
 * @brief Присоединение карты освещения
 * @ingroup material
 */
void sMaterialSetLightMap(sMaterialID, sTextureID);


/**
 * @brief Активация материала, который будет
 * использован при растеризации полисетки.
 * 
 * @ingroup material
 */
bool sMaterialBind(sMaterialID material);


/**
 * @brief Удаление материала из списка пользователей
 * @ingroup material
 */
void sMaterialRemoveUser(sMaterialID material, sMeshID mesh);


/**
 * @brief Очистка списка пользователей материала
 * @ingroup material
 */
void sMaterialRemoveUsers(sMaterialID material);


/**
 * @brief Вывод всех пользователей текстуры
 * @ingroup texture
 */
void sTexturePrintUsers(sTextureID tex);


/**
 * @brief Общее количество материалов
 * @ingroup material
 */
size_t sMaterialGetQuantity(void);


/**
 * @brief Очистка неиспользуемых материалов
 * @ingroup material
 */
void sMaterialClear(void);

#ifdef __cplusplus
}
#endif

#endif
