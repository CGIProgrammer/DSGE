/*
 * framebuffer.h
 *
 *  Created on: 15 июль 2020 г.
 *      Author: Ivan G
 */

#ifndef FRAMEBUFFER_H
#define FRAMEBUFFER_H

#include "structures/types.h"
#include "structures/texture.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @defgroup framebuffer sFrameBuffer
 * 
 * Фреймбуфер с целями рендера (render targets).
 * В нём можно хранить результаты выполнения шейдеров. Эти
 * результаты можно использовать в качестве обычных текстур
 * в составе обычных материалов.
 * Полезно для постпроцессинга или обработки текстур.
 * 
 * @todo Оптимизировать использование буферов глубины:
 * сделать так, чтобы при создании новых буферов FBO создавались
 * только если их ещё нет с таким разрешением.
 */


/**
 * @brief Настройка стандартного фреймбуфера
 * @param width Ширина
 * @param height Высота
 */
void sFrameBufferSetStd(uint16_t width, uint16_t height);


/**
 * @brief Получение стандартного фреймбуфера
 */
sFrameBuffer sFrameBufferGetStd(void);

/**
 * @brief Создание нового фреймбуфера
 * @param width Ширина
 * @param height Высота
 * @param depthbuffer Использование буфера глубины.
 * Необходимо для теста глубины, который, в свою очередь, необходим
 * для того, чтобы ближние треугольники не перекрывались дальними.
 */
sFrameBuffer sFrameBufferCreate(uint16_t width, uint16_t height, bool depthbuffer);


/**
 * @brief Добавление буфера глубины к фреймбуферу
 * Необходимо для теста глубины, который, в свою очередь, необходим
 * для того, чтобы ближние треугольники не перекрывались дальними.
 * 
 * @param fb Фреймбуфер, которому будет добавлен буфер глубины
 */
void sFrameBufferAddDepth(sFrameBufferID fb);


/**
 * @brief Удаление буфера глубины у фреймбуфера
 */
void sFrameBufferRemoveDepth(sFrameBufferID fb);


/**
 * @brief Добавление цели рендера.
 * Цель рендера - текстура, в которую будет производиться выбор шейдера
 * 
 * @param fb Фреймбуфер, которому добавляется текстура
 * @param texture Добавляемая текстура
 * @ingroup frame_buffer
 */
void sFrameBufferAddRenderTarget(sFrameBufferID fb, sTextureID texture);


/**
 * @brief Удаление текстуры из целей рендера.
 * 
 * @param fb Фреймбуфер, от которого отсоединяется текстура
 * @param texture Отсовединяемая текстура
 * @ingroup frame_buffer
 */
void sFrameBufferRemoveRenderTarget(sFrameBufferID fb, sTextureID texture);


/**
 * @brief Удаление текстуры из целей рендера по индексу.
 * 
 * @param fb Фреймбуфер, от которого отсоединяется текстура
 * @param texture Индекс отсоединяемой текстуры
 * @ingroup frame_buffer
 */
void sFrameBufferRemoveRenderTargetIndex(sFrameBufferID fb, int texture);


/**
 * @brief Присоединение текстуры в качестве буфера глубины.
 * @param fb Фреймбуфер, которому присоединяется текстура
 * @param texture Присоединяемая текстура
 * @ingroup frame_buffer
 */
void sFrameBufferSetDepthTarget(sFrameBufferID fb, sTextureID texture);


/**
 * @brief Установка буфера, в который будет выполняться рендеринг.
 * @param fb Устанавливаемый фреймбуфер. Если null, то рендеринг будет производиться
 * прямо в окно приложения
 * 
 * @param textures Набор битов, указывающих на то, какие текстуры нужно привязать
 * в качестве целей рендеринга (например 0b1010 означает, что нужно привязать 
 * текстуры под индексами 1 и 3)
 * 
 * @ingroup frame_buffer
 */
void sFrameBufferBind(sFrameBufferID fb, uint16_t textures, bool bind_depth);


/**
 * @brief Заполнение активного фреймуфера цветом.
 * @param rgba Цвет, которым будет заполняться фреймбуфер.
 * @ingroup frame_buffer
 */
void sFrameBufferFillColor(sColor rgba);


/**
 * @brief Удаление буфера. Освобождение памяти, выделенной под фреймбуфер.
 * @param fb Устанавливаемый фреймбуфер. Если null, то рендеринг будет производиться
 * прямо в окно приложения
 */
void sFrameBufferDelete(sFrameBufferID fb);

#ifdef __cplusplus
}
#endif

#endif
