#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include "structures/list.h"
#include "memmanager.h"
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

void listPushBack(void** list, void* item, size_t size)
{
	size_t oldSize = sSizeof((void*)*list);
	size_t newSize = oldSize + size;
	//if (newSize>sSizeof(*list)) {
		*list = sRealloc(*list, newSize);
	//}
	memmove((void*)(((uintptr_t)*list) + oldSize), item, size);
}

void listPopItem(void** list, size_t index, size_t size, void* destination)
{
	void*** ptr = (void***)list;
	if (index==(size_t)-1) return;
	size_t length = sSizeof(*ptr) / size;
	if (index>=length && !length) return;
	
	void* source = (void*)(((uintptr_t)*list) + (index+1)*size);
	void* destin = (void*)(((uintptr_t)*list) + index*size);
	if (destination)
	{
		memmove(destination, destin, size);
	}
	if (length>1)
	{
		memmove(destin, source, (length-index-1)*size);
		*ptr = (void**)sRealloc((void*)*ptr, (length-1)*size);
	}
	else
	{
		sDelete(*ptr);
	}
}

void listPop(void** list, size_t index)
{
	void*** ptr = (void***)list;
	size_t length = sListGetSize(*ptr);
	
	if (index >= length || !length) return;

	if (length == 1)
	{
		sFree(*list);
		*list = 0;
	}
	else
	{
		void** ptr = (void**)*list;
		for (size_t i=index; i<length-1; i++)
		{
			ptr[i] = ptr[i+1];
		}
		size_t newSize = sizeof(void*)*(length-1);
		//if (newSize < sSizeof(*list) / 2) {
			*list = sRealloc(*list, newSize);
		//}
	}
}

void listPopPointer(void** list, void* pointer)
{
	void*** ptr = (void***)list;
	size_t length = sListGetSize(*ptr);
	if (!length)
	{
		return;
	}
	size_t index = listIndexOf(*ptr, pointer);
	for (index = 0; index<length && (*ptr)[index]!=pointer; index++);
	if (index+1==0)
	{
		return;
	}
	listPop(list, index);
}

size_t listIndexOf(void** list, void* item)
{
	void*** ptr = (void***)list;
	size_t length = sListGetSize(*ptr);
	if (!length)
	{
		return MAX_INDEX;
	}
	for (size_t i=0;i<length;i++)
	{
		if (list[i] == item)
		{
			return i;
		}
	}
	return MAX_INDEX;
}

void listSwapElements(void** list, size_t ind1, size_t ind2)
{
	void* w = list[ind1];
	list[ind1] = list[ind2];
	list[ind2] = w;
}

size_t listGetIndexOf(void** list, void* item, size_t size)
{
	void *ptr = *list;
	size_t length = sSizeof(ptr) / size;
	for (size_t i=0; i<length; i++)
	{
		if (!memcmp((void*)((uintptr_t)ptr+i*size), item, size)) {
			return i;
		}
	}
	return MAX_INDEX;
}

size_t staticListIndexOf(void** list, size_t list_length, void* item, size_t item_size)
{
	for (size_t i=0; i<list_length; i++)
	{
		if (!memcmp((void*)((uintptr_t)(*list) + i * item_size), item, item_size)) {
			return i;
		}
	}
	return MAX_INDEX;
}

#ifdef __cplusplus
}
#endif