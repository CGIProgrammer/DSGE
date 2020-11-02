#ifndef DICT_H
#define DICT_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif


/**
 * @defgroup dict sDict
 *
 * Словарь, реализованный в виде хэш-таблицы c автоматическим
 * расширением
 */

typedef struct {
    uint64_t key_hash;
    void* key;
    void* value;
} sHashTableCell;

typedef struct {
    sHashTableCell** cells;
    size_t pairs_count;
} sDict;
/*
typedef struct {
    void*** keys;
    void*** values;
    size_t pairs_count;
} sDict;*/


/**
 * @brief Создание словаря
 * @param table_size Изначальный размер хэш-таблицы
 * @ingroup dict
 */
sDict sDictNew(size_t table_size);


/**
 * @brief Проверка наличия элемента в словаре
 * @param dict Указатель на словарь
 * @param key Указатель на данные ключа
 * @param key_size Размер ключа
 * @param index_a Сюда будет записан индекс списка с элементом (может быть нулём)
 * @param index_b Сюда будет записан индекс элемента в списке (может быть нулём)
 * @ingroup dict
 */
uint8_t sDictHaveItem(sDict* dict, void* key, size_t key_size, size_t* index_a, size_t* index_b, uint32_t* hash);


/**
 * @brief Получение элемента из словаря
 * @ingroup dict
 */
void* sDictGetItem(sDict* dict, void* key, size_t key_size);


/**
 * @brief Добавление указателя в словарь
 * @param item Указатель, который будет добавлен
 * @ingroup dict
 */
void sDictAddItem(sDict* dict, void* key, size_t key_size, void* item);


/**
 * @brief Размер хэш-таблицы
 * @ingroup dict
 */
size_t sDictGetTableSize(sDict* dict);


/**
 * @brief Удаляение словаря
 * @ingroup dict
 */
void sDictDelete(sDict* dict);


/**
 * @brief Изменение размера хэш-таблицы (с схоранением данных)
 * @param table_size Размер хэш-таблицы
 * @ingroup dict
 */
void sDictRealloc(sDict* dict, size_t table_size);


/**
 * @brief Удаление элемента
 * @ingroup dict
 */
void sDictRemoveItem(sDict* dict, void* key, size_t key_size);


/**
 * @brief Удаление элемента по значению, а не по ключу
 * @param dict Указатель на словарь
 * @param key Значение
 * @ingroup dict
 */
void sDictRemoveItemByValue(sDict* dict, void* key);


/**
 * @brief ДОбавление указателя по строковому ключу
 * @param dict Словарь
 * @param key Строковый ключ
 * @param item Добавляемый указатель
 */
void sDictAddItemKW(sDict* dict, char* key, void* item);


/**
 * @brief Удаленние указателя по строковому ключу
 * @param dict Словарь
 * @param key Строковый ключ
 */
void sDictRemoveItemKW(sDict* dict, char* key);


/**
 * @brief Получение указателя по строковому ключу
 * @param dict Словарь
 * @param key Строковый ключ
 * @return Возвращает указатель, который соответствует
 * ключу, или null, если такого ключа нет
 */
void* sDictGetItemKW(sDict* dict, char* key);


/**
 * @brief Проверка наличия в словаре строкового ключа
 * @param dict Словарь
 * @param key Строковый ключ
 */
uint8_t sDictHaveItemKW(sDict* dict, char* key);

#ifdef __cplusplus
}
#endif

#endif