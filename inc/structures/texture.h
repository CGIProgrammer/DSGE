
/*
 * shader.h
 *
 *  Created on: 9 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef TEXTURE_H
#define TEXTURE_H

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "types.h"
#include "memmanager.h"
#include "structures/material.h"
#include "structures/framebuffer.h"
#include "structures/list.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup texture sTexture
 * 
 * Изображение-текстура, которая может быть
 * @li плоским изображением (2D текстура)
 * @li воксельной моделью (3D текстура)
 * @li кубической панорамой
 * @li массивом 2D текстур
 */

typedef struct sTexture
{
	char name[256];
	uint16_t width;
	uint16_t height;
	void* data;
	uint32_t type;
	uint32_t color_format;
	GLuint ID;
	bool fake_user;
	sTextureID* sides;
	sTextureID parent;
    sMaterialID* material_users;
    sFrameBufferID* framebuffer_users;
} sTexture;

#ifndef glc
#define glc(func) func
#endif

static const int sTextureFormatTable[][3] = {
	{GL_RED,	GL_R8,		GL_UNSIGNED_BYTE},
	{GL_RED,	GL_R16,		GL_UNSIGNED_SHORT},
	{GL_RED,	GL_R32UI,	GL_UNSIGNED_INT},
	{GL_RED,	GL_R16F,	GL_HALF_FLOAT},
	{GL_RED,	GL_R32F,	GL_FLOAT},
	{GL_RG,		GL_RG8,		GL_UNSIGNED_BYTE},
	{GL_RG,		GL_RG16,	GL_UNSIGNED_SHORT},
	{GL_RG,		GL_RG32UI,	GL_UNSIGNED_INT},
	{GL_RG,		GL_RG16F,	GL_HALF_FLOAT},
	{GL_RG,		GL_RG32F,	GL_FLOAT},
	{GL_RGB,	GL_RGB8,	GL_UNSIGNED_BYTE},
	{GL_RGB,	GL_RGB16,	GL_UNSIGNED_SHORT},
	{GL_RGB,	GL_RGB32UI,	GL_UNSIGNED_INT},
	{GL_RGB,	GL_RGB16F, 	GL_HALF_FLOAT},
	{GL_RGB,	GL_RGB32F, 	GL_FLOAT},
	{GL_RGBA,	GL_RGBA8,  	GL_UNSIGNED_BYTE},
	{GL_RGBA,	GL_RGBA16, 	GL_UNSIGNED_SHORT},
	{GL_RGBA, 	GL_RGBA32UI,GL_UNSIGNED_INT},
	{GL_RGBA,	GL_RGBA16F, GL_HALF_FLOAT},
	{GL_RGBA,	GL_RGBA32F, GL_FLOAT},
	{GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT16, GL_UNSIGNED_SHORT},
	{GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT32, GL_UNSIGNED_INT},
	{GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT32F, GL_FLOAT}
};

typedef enum
{
	GRAY8I = 0,
	GRAY16I,
	GRAY32I,
	GRAY16F,
	GRAY32F,

	RED8I = 0,
	RED16I,
	RED32I,
	RED16F,
	RED32F,

	RG8I,
	RG16I,
	RG32I,
	RG16F,
	RG32F,

	RGB8I,
	RGB16I,
	RGB32I,
	RGB16F,
	RGB32F,

	RGBA8I,
	RGBA16I,
	RGBA32I,
	RGBA16F,
	RGBA32F,

	SHADOW16I,
	SHADOW32I,
	SHADOW32F
} sTexturePixFmt;

typedef struct
{
	uint32_t format;
	uint32_t signature;
	uint32_t height;
	uint32_t width;
	uint32_t mipMapNumber;
	uint32_t formatCode;
	uint32_t blockSize;
	uint32_t offset;
	void *dataPtr;
} DDS_DATA;

typedef enum {
	sTextureRepeat,
	sTextureRepeatMirror,
	sTextureClampEdge,
	sTextureClampBlack
} sTextureSamplingMode;

/**
 * @brief Создание новой текстуры
 * @param name Имя
 * @return Возвращает указатель на новую текстуру (sTextureID)
 * @ingroup texture
 */
sTextureID sTextureCreateEmpty(char* name);


/**
 * @brief Удаление текстуры
 * @param texture Указатель на текстуру
 * @ingroup texture
 */
void sTextureDelete(sTextureID texture);


/**
 * @brief Загрузка сжатой текстуры из файла
 * @param name Путь к файлу
 * @return Возвращает указатель на загруженную текстуру
 * @ingroup texture
 */
sTextureID sTextureLoadDDS(char* name);


/**
 * @brief Загрузка сжатой кубической текстуры из файла
 * @param name Путь к файлу
 * @return Возвращает указатель на загруженную текстуру
 * @ingroup texture
 */
sTextureID sTextureLoadDDSCubemap(const char* name);


/**
 * @brief Разделение кубической текстуры на плоские текстуры.
 * Каждая кубическая текстура состоит на 6 плоских текстур
 * на каждую из граней куба.
 * 
 * @param cubemap Указатель на кубическую текстуру
 * @ingroup texture
 */
void sTextureCubeSplit(sTextureID cubemap);

/**
 * @brief Загрузка сжатой текстуры из данных по указателю
 * @param name Имя, присваемое создаваемой текстуре
 * @param data Данные в формате DDS
 * @return Возвращает указатель на загруженную текстуру
 * @ingroup texture
 */
sTextureID sTextureLoadDDSFromMem(char* name, void* data);


/**
 * @brief Создание пустой кубической текстуры
 * @param name Имя, присваемое создаваемой текстуры
 * @param width Ширина стороны
 * @param height Высота стороны
 * @param pix_fmt Формат пикселя
 * @param filtering Использование фильтрации
 * @param mipmaps Использование mip текстур
 * @return Возвращает указатель на новую текстуру
 * @ingroup texture
 */
sTextureID sTextureCreateCubemap(char* name, uint16_t width, uint16_t height, sTexturePixFmt pix_fmt, bool filtering, bool mipmaps);


/**
 * @brief Создание пустой плоской текстуры
 * @param name Имя, присваемое создаваемой текстуры
 * @param width Ширина стороны
 * @param height Высота стороны
 * @param pix_fmt Формат пикселя
 * @param filtering Использование фильтрации
 * @param mipmaps Использование mip текстур
 * @param data Данные
 * @return Возвращает указатель на новую текстуру
 * @ingroup texture
 */
sTextureID sTextureCreate2D(char* name, uint16_t width, uint16_t height, sTexturePixFmt pix_fmt, bool filtering, bool mipmaps, void* data);


/**
 * @brief Генерация mip уровней текстуры
 * @param texture Сохраняемая текстура
 * @ingroup texture
 */
void sTextureGenerateMipMaps(sTextureID texture);

/**
 * @brief Сохранение текстуры в файл
 * @param texture Сохраняемая текстура
 * @param fname Имя файла. Если null, то название берётся из имени текстуры
 * @ingroup texture
 */
void sTextureSave(sTextureID texture, const char* fname);


/**
 * @brief Загрузка текстуры из файла
 * @param fname Имя файла
 * @param tname Имя текстуры. Если null, то название берётся из имени файла
 * @ingroup texture
 */
sTextureID sTextureLoad(const char* fname, const char* tname);


/**
 * @brief Установка режима наложения текстуры
 * 
 * @param texture Текстура
 * @param tiling Режим наложения
 * sTextureRepeat - наложение с повторением (как плитка)
 * sTextureRepeatMirror - наложение с повторением и отражением
 * sTextureClampEdge - наложение с отсечением границы
 * sTextureClampBlack - наложение с чёрными краями
 * 
 * @ingroup texture
 */
void sTextureSetTiling(sTextureID texture, sTextureSamplingMode tiling);


/**
 * @brief Создание текстуры с "белым" шумом.
 * Полезно для создания облаков, дыма и т.д.
 * 
 * @param seed Задающее число для генератора случайных чисел
 * @param width Ширина текстуры
 * @param height Высота текстуры
 * @return Возвращает указатель текстуру (массив) с синим щумом
 * @ingroup texture
 */
sTextureID sTextureGenerateWhiteNoise(int seed, int w, int h);


/**
 * @brief Создание текстуры с "синим" (высокочастотным) шумом.
 * Полезно для различных эффектов типа SSAO или матового отражения.
 * 
 * @param width Ширина текстуры
 * @param height Высота текстуры
 * @return Возвращает указатель текстуру (массив) с синим щумом
 * @ingroup texture
 */
sTextureID sTextureGenerateBlueNoise(int w, int h);


/**
 * @brief Получение общего количества текстур
 * @ingroup texture
 */
size_t sTextureGetQuantity(void);


/**
 * @brief Добавление фреймбуфера в список пользователей
 * @ingroup texture
 */
void sTextureAddFramebufferUser(sTextureID texture, sFrameBufferID framebuffer);


/**
 * @brief Добавление материала в список пользователей
 * @ingroup texture
 */
void sTextureAddMaterialUser(sTextureID texture, sMaterialID material);


/**
 * @brief Удаление фреймбуфера из списка пользователей
 * @ingroup texture
 */
void sTextureRemoveFramebufferUser(sTextureID texture, sFrameBufferID framebuffer);


/**
 * @brief Удаление материала из списка пользователей
 * @ingroup texture
 */
void sTextureRemoveMaterialUser(sTextureID texture, sMaterialID material);


/**
 * @brief Очистка списка пользователей текстуры
 * @ingroup texture
 */
void sTextureRemoveUsers(sTextureID texture);

/**
 * @brief Вывод всех пользователей текстуры
 * @ingroup texture
 */
void sTexturePrintUsers(sTextureID tex);


/**
 * @brief Удаление неиспользуемых текстур
 * @ingroup texture
 */
void sTextureClear(void);

#ifdef __cplusplus
}
#endif

#endif
