
/*
 * shader.h
 *
 *  Created on: 9 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef SHADER_H
#define SHADER_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "structures/types.h"
#include "memmanager.h"
#include "structures/texture.h"
#include "structures/dict.h"

#define MAX_LIGHTS 4

/**
 * @defgroup shader sShader
 * 
 * Набор функций для загрузки шейдеров.
 * Стандартный препроцессор GLSL дополнен директивой #include.
 * 
 * @warning Реализованная директива include не подпадает под
 * директивы #ifdef, #ifndef и так далее. То есть, include в
 * любом случае будет заменён на содержимое указанного файла.
 * Поэтому пока что рекурсивное включение запрещено.
 * 
 * @todo Доработать препроцессор
 */


typedef enum {
	lSunColor = 0,
	lSunTransform,
	lSunDepth,
	lSunShadowMap,
	lSunVarCount,

	lSpotColor = 0,
	lSpotTransform,
	lSpotBlending,
	lSpotAngle,
	lSpotShadowmap,
	lSpotPosition,
	lSpotVarCount,

	lPointPosition = 0,
	lPointColor,
	lPointShadowmap,
	lPointVarCount,

	vObjectTransform = 0,
	vObjectTransformPrev,
	vCameraTransform,
	vCameraTransformStable,
	vCameraTransformPrev,

	vCameraProjection,
	vCameraProjectionInv,
	lSpotCount,
	lPointCount,
	fDistanceFormat,
	fDiffuseMap,
	fSpecularMap,
	fReliefMap,
	fLightMap,
	fMetallicMap,
	fRoughnessMap,
	fDiffuseValue,
	fSpecularValue,
	fReliefValue,
	fMetallicValue,
	fRoughnessValue,
	fFresnelValue,
	fTexScrollX,
	fTexScrollY,
	gSize,
	gPosition,
	gResolution,
	vfVarCount
} sShaderUniformLocation;

typedef struct sShader
{
	GLchar* fragment_source;
	GLchar* vertex_source;
	GLint   log_len;
	GLint   frag_source_len;
	GLint   vert_source_len;
	GLint   success;
	GLuint  fragment_id;
	GLuint  vertex_id;
	GLuint  program_id;
	GLuint  log;
    sDict   uniform_cache;
    uint8_t texture_count;
    uint8_t point_light_count;
	uint8_t spotlight_count;
	GLuint base_vars[vfVarCount];
	GLuint point_light_vars[MAX_LIGHTS][lPointVarCount];
	GLuint spotlight_vars[MAX_LIGHTS][lSpotVarCount];
	GLuint sunlight_vars[lSunVarCount];
	FILE*  fp;
	bool fake_user;
	sMaterialID* material_users;
	sCameraComponentID* render_users;
} sShader;

#ifndef glc
#define glc(func) {func;}
#endif

#ifdef __cplusplus
extern "C" {
#endif

#include "structures/types.h"
#include "structures/material.h"
#include "structures/components/camera_component.h"


/**
 * @brief Загрузка шейдеров из файлов.
 * Файлы не компилируются повторно. Например:
 * sShaderID phong = sShaderMakeFromFiles("vert.glsl", "frag_phong.glsl");
 * sShaderID toon = sShaderMakeFromFiles("vert.glsl", "frag_toon.glsl");
 * vert.glsl скомпилируется только один раз. При втором вызове
 * будет идентификатор вершинного шейдера "vert.glsl" будет использован
 * повторно.
 * 
 * @param name_vert Имя файла с вертексным шейдером
 * @param name_frag Имя файла с фрагментным шейдером
 * @return Возвращает указатель на новый шейдер (sShaderID)
 * @ingroup shader
 */
sShaderID sShaderMakeFromFiles(const char* name_vert, const char* name_frag);


/**
 * @brief Загрузка шейдера, описывающего материал.
 * Загружает файл с функцией void pbr(), который задаёт
 * параметры материала. Может применяться для процедурных материалов
 * 
 * @param file_name Имя файла с шейдером
 * @param base Выход основного шейдера
 * @param skeleton Выход шейдера для полисеток со скелетной деформацией
 * @param base_shadow Выход основного шейдера для теней
 * @param skeleton_shadow Выход шейдера для полисеток со скелетной деформацией (для теней)
 * @ingroup shader
 */
void sShaderLoadHL(
	char* file_name, 
	sShaderID* base,
	sShaderID* skeleton,
	sShaderID* base_shadow,
	sShaderID* skeleton_shadow
);


/**
 * @brief Активировать шейдер.
 * Активированный шейдер будет использоваться при вызове отрисовки.
 * Обнуляет счётчик привязанных текстур.
 * 
 * @param shader Активируемый шейдер
 * @returns Возвращаяет 0, если функция вызвана повторно с
 * тем же шейдером
 * @ingroup shader
 */
bool sShaderBind(sShaderID shader);


/**
 * @brief Добавление материала, который буде зависеть 
 * от указанного шейдера
 * 
 * @param shader Шейдер, от которого будет зависеть материал
 * @param material Зависимый материал
 * @ingroup shader
 */
void sShaderAddMaterialUser(sShaderID shader, sMaterialID material);


/**
 * @brief Добавление компонента камеры, который будет зависеть 
 * от указанного шейдера
 * 
 * @param shader Шейдер, от которого будет зависеть материал
 * @param renderer Компонент камеры
 * @ingroup shader
 */
void sShaderAddRenderbufferUser(sShaderID shader, sCameraComponentID renderer);


/**
 * @brief Отсоединение зависимого материала
 * 
 * @param shader Шейдер, от которого будет отсоединён материал
 * @param material Зависимый материал
 * @ingroup shader
 */
void sShaderRemoveMaterialUser(sShaderID shader, sMaterialID material);


/**
 * @brief Отсоединение зависимого компонента камеры
 * 
 * @param shader Шейдер, от которого будет отсоединён компонент камеры
 * @param material Зависимый компонент камеры
 * @ingroup shader
 */
void sShaderRemoveRenderbufferUser(sShaderID shader, sCameraComponentID renderer);


/**
 * @brief Привязать текстуру к активному шейдеру
 * Увеличивает значение счётчика привязанных текстур на 1
 * Максимум можно привязать 32 текстуры (ограничение OpenGL)
 * 
 * @param shader Шейдер, к которому будет привязана текстура
 * @param texture Текстура, которая будет привязана
 * @param name Название переменной шейдера, которой будет назначена текстура
 * @ingroup shader
 */
bool sShaderBindTexture(sShaderID shader, char* name, sTextureID texture);
bool sShaderBindTextureToID(sShaderID shader, GLuint id, sTextureID texture);


/**
 * @brief Передать шейдеру массив плавающих чисел
 * Подходит как для векторов так и для матриц.
 * Размер массива должен быть в интервале от 1 до 16 чисел включительно.
 * 
 * @param shader Шейдер, которому будет передан массив
 * @param name Название переменной шейдера, которой будет назначен массив
 * @param data Указатель на передаваемый массив
 * @param count Количество элементов в массиве
 * @ingroup shader
 */
void sShaderBindUniformFloatArray(sShaderID shader, char* name, float* data, uint32_t count);
void sShaderBindUniformFloatArrayToID(sShaderID shader, GLuint id, float* data, uint32_t count);


/**
 * @brief Передать шейдеру число с плавающей точкой
 * 
 * @param shader Шейдер, которому будет передано число
 * @param name Название переменной шейдера, которой будет присвоено число
 * @param value Передаваемое число
 * @ingroup shader
 */
void sShaderBindUniformFloat (sShaderID shader, char* name, float value);
void sShaderBindUniformFloatToID (sShaderID shader, GLuint id, float value);


/**
 * @brief Передать шейдеру 2D вектор с плавающими числами
 * 
 * @param shader Шейдер, которому будет передан вектор
 * @param name Название переменной шейдера, которой будет присвоен вектор
 * @ingroup shader
 */
void sShaderBindUniformFloat2(sShaderID shader, char* name, float x, float y);
void sShaderBindUniformFloat2ToID(sShaderID shader, GLuint id, float x, float y);


/**
 * @brief Передать шейдеру 3D вектор с плавающими числами
 * 
 * @param shader Шейдер, которому будет передан вектор
 * @param name Название переменной шейдера, которой будет присвоен вектор
 * @ingroup shader
 */
void sShaderBindUniformFloat3(sShaderID shader, char* name, float x, float y, float z);
void sShaderBindUniformFloat3ToID(sShaderID shader, GLuint id, float x, float y, float z);


/**
 * @brief Передать шейдеру XYZW вектор с плавающими числами
 * 
 * @param shader Шейдер, которому будет передан вектор
 * @param name Название переменной шейдера, которой будет присвоен вектор
 * @ingroup shader
 */
void sShaderBindUniformFloat4(sShaderID shader, char* name, float x, float y, float z, float w);
void sShaderBindUniformFloat4ToID(sShaderID shader, GLuint id, float x, float y, float z, float w);


/**
 * @brief Передать шейдеру целое число
 * 
 * @param shader Шейдер, которому будет передано число
 * @param name Название переменной шейдера, которой будет присвоено число
 * @param value Передаваемое число
 * @ingroup shader
 */
void sShaderBindUniformInt (sShaderID shader, char* name, int value);
void sShaderBindUniformIntToID (sShaderID shader, GLuint id, int value);


/**
 * @brief Передать шейдеру 2D вектор с целыми числами
 * 
 * @param shader Шейдер, которому будет передан вектор
 * @param name Название переменной шейдера, которой будет присвоен вектор
 * @ingroup shader
 */
void sShaderBindUniformInt2(sShaderID shader, char* name, int x, int y);
void sShaderBindUniformInt2ToID(sShaderID shader, GLuint id, int x, int y);


/**
 * @brief Передать шейдеру 3D вектор с целыми числами
 * 
 * @param shader Шейдер, которому будет передан вектор
 * @param name Название переменной шейдера, которой будет присвоен вектор
 * @ingroup shader
 */
void sShaderBindUniformInt3(sShaderID shader, char* name, int x, int y, int z);
void sShaderBindUniformInt3ToID(sShaderID shader, GLuint id, int x, int y, int z);


/**
 * @brief Передать шейдеру XYZW вектор с целыми числами
 * 
 * @param shader Шейдер, которому будет передан вектор
 * @param name Название переменной шейдера, которой будет присвоен вектор
 * @ingroup shader
 */
void sShaderBindUniformInt4(sShaderID shader, char* name, int x, int y, int z, int w);
void sShaderBindUniformInt4ToID(sShaderID shader, GLuint id, int x, int y, int z, int w);


/**
 * @brief Передать структуру источника света
 * 
 * @param shader Шейдер, которому будет передана структура
 * @param light_object Объект, с которого будет передана структура
 * @ingroup shader
 */
void sShaderBindLight(sShaderID shader, sGameObjectID light_object);
void sShaderBindLights(sShaderID shader, sGameObjectID* lights);


/**
 * @brief Передать устые структуры источников света
 *
 * @param shader Шейдер, которому будет передана структура
 * @param light_object Объект, с которого будет передана структура
 * @ingroup shader
 */
void sShaderUnbindLights(sShaderID shader);


/**
 * @brief Очистка списков пользователей шейдера
 * @ingroup shader
 */
void sShaderRemoveUsers(sShaderID shader);


/**
 * @brief Удаление шейдера
 * @param shader Удаляемый шейдер
 * @ingroup shader
 */
void sShaderDelete(sShaderID shader);


/**
 * @brief Удаление неиспользуемых шейдеров
 * @ingroup shader
 */
void sShaderClear(void);

void sShaderDeleteDict(void);

void sShaderSetVersion(const char* version);

#ifdef __cplusplus
}
#endif

#endif
