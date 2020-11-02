#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include "memmanager.h"

#ifdef __cplusplus
extern "C" {
#endif

static size_t bytes_allocated = 0;

void* sCalloc(size_t size,size_t count)
{
    size_t block = size*count+sizeof(size_t);
    bytes_allocated += block;
    void* ptr = calloc(block,1);
    *(size_t*)ptr = block;
    return (void*)((uintptr_t)ptr + sizeof(size_t));
}

void* sMalloc(size_t size)
{
    size += sizeof(size_t);
    bytes_allocated += size;
    void* ptr = malloc(size);
    *(size_t*)ptr = size;
    return (void*)((uintptr_t)ptr + sizeof(size_t));
}

void* sRealloc(void* old_ptr,size_t size)
{
    if (!old_ptr)
    {
        return sMalloc(size);
    }
    size+=sizeof(size_t);
    void* new_ptr;
    bytes_allocated += size - *(size_t*)((intptr_t)old_ptr - sizeof(size_t));
    new_ptr = realloc((void*)((intptr_t)old_ptr - sizeof(size_t)),size);
    *(size_t*)new_ptr = size;
    return (void*)((uintptr_t)new_ptr + sizeof(size_t));
}

void* sRecalloc(void* old_ptr,size_t size)
{
    if (!old_ptr)
    {
        return sCalloc(size,1);
    }
    uint64_t previous_size = sSizeof(old_ptr);
    size+=sizeof(size_t);
    void* new_ptr;
    bytes_allocated += size - previous_size;
    new_ptr = realloc((void*)((intptr_t)old_ptr - sizeof(size_t)),size);
    *(size_t*)new_ptr = size;
    memset((void*)((uintptr_t)new_ptr + previous_size), 0, size-previous_size);
    return (void*)((uintptr_t)new_ptr + sizeof(size_t));
}

void sFree(void* ptr)
{
	if (!ptr)
	{
		return;
	}
    bytes_allocated -= *(size_t*)((uintptr_t)ptr-sizeof(size_t));
    free((void*)((uintptr_t)ptr-sizeof(size_t)));
}

void sFree2(void** ptr)
{
	if (!ptr)
	{
		return;
	}
    for (int i=0; i<sSizeof(ptr)/sizeof(uintptr_t); i++)
    {
        sFree(ptr[i]);
    }
    bytes_allocated -= *(size_t*)((uintptr_t)ptr-sizeof(size_t));
    free((void*)((uintptr_t)ptr-sizeof(size_t)));
}

size_t sSizeof(void* ptr)
{
    if (!ptr) return 0;
	return *(size_t*)((uintptr_t)ptr-sizeof(size_t)) - sizeof(size_t);
}

size_t sGetAllocatedMem(void)
{
    return bytes_allocated;
}

#ifdef __cplusplus
}
#endif