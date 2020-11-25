#ifndef LIST_H
#define LIST_H

#include <stdint.h>
#include <stddef.h>

/**
 * @defgroup list sList
 * 
 * Набор функций и макросов для упрощения работы с динамическими массивами.
 * Операции удаления из массива сопровождаются сдвигом и перераспределением памяти,
 * поэтому значение указателя может меняться
 * 
 * @todo Сделать так, чтобы память перераспределялась не при каждом добавлении и удалении,
 * тем самым увеличить быстродействие.
 * 
 * @warning Внимание, указатель на массив должен быть создан при помощи функции
 * sMalloc или sCalloc, либо иметь значение null.
 */


/**
 * @brief Получение количество элементов массива.
 * @param list Указатель на массив
 * @ingroup list
 */
#define sListGetSize(list) (sSizeof(list) / sizeof(*(list)))


/**
 * @brief Добавление элемента в конец массива.
 * Меняет размер массива, может изменить указатель.
 * @param list Указатель на массив
 * @ingroup list
 */
#define sListPushBack(list, item) listPushBack((void**)&(list), &(item), sizeof(item))


/**
 * @brief Линейный поиск элемента в массиве
 * @param list Указатель на массив
 * @param item Элемент поиска
 * @return Возвращает индекс элемента или -1, если такого элемента нет
 * @ingroup list
 */
#define sListIndexOf(list, item) listGetIndexOf((void**)&(list), &(item), sizeof(item))


/**
 * @brief Поиск и удаление указателя из массива.
 * Очень не рекомендуется его использовать. Вместо него
 * чучше использовать sListPopItem
 * @param list Указатель на массив
 * @param ptr Указатель, который надо удалить из массива
 * @ingroup list
 */
#define sListPopPtr(list, ptr) listPopPointer((void**)&(list), &(ptr))


/**
 * @brief Удаление указателя из массива по индексу.
 * Очень не рекомендуется его использовать. Вместо него
 * чучше использовать sListPopIndex
 * @param list Указатель на массив
 * @param ptr_index Индекс элемента, который надо удалить из массива
 * @ingroup list
 */
#define sListPopPtrIndex(list, ptr_index) listPop((void**)&(list), (ptr_index))


/**
 * @brief Поиск и удаление элемента из массива.
 * @param list Указатель на массив
 * @param ptr Элемент, который надо удалить из массива
 * @ingroup list
 */
#define sListPopItem(list, item)   listPopItem((void**)&(list), listGetIndexOf((void**)&(list), &(item), sizeof(item)), sizeof(item), 0)


/**
 * @brief Удаление элемента из массива по индексу.
 * @param list Указатель на массив
 * @param ptr_index Индекс элемента, который надо удалить из массива
 * @ingroup list
 */
#define sListPopIndex(list, index) listPopItem((void**)&(list), index, sizeof(*(list)), 0)


/**
 * @brief Получение индекса элемента в статично массиве.
 * @param list Указатель на массив
 * @param list_length Длина массива
 * @param item Элемент, индекс которого надо найти
 * @ingroup list
 */
#define sStaticListIndexOf(list, list_length, item) staticListIndexOf((void**)&(list), list_length, &item, sizeof(item))

#define sListSwap(list, ind1, ind2) listSwapElements((void**)&(list), ind1, ind2)
#define MAX_INDEX SIZE_MAX

#ifdef __cplusplus
extern "C" {
#endif

void listPushBack(void** list, void* item, size_t size);
void listPop(void** list, size_t index);
void listPopPointer(void** list, void* pointer);
size_t listIndexOf(void** list, void* item);
size_t listGetIndexOf(void** list, void* item, size_t size);
void listSwapElements(void** list, size_t ind1, size_t ind2);
void listPopItem(void** list, size_t index, size_t size, void* destination);
size_t staticListIndexOf(void** list, size_t list_length, void* item, size_t item_size);

#ifdef __cplusplus
}
#endif

#endif