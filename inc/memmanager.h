#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>

#ifndef MEMMANAGER_H
#define MEMMANAGER_H

#define sNew(type) ((type*)sCalloc(sizeof(type), 1))
#define sNewArray(type, count) ((type*)sCalloc(sizeof(type), count))
#define sDelete(obj) {sFree((void*)(obj)); obj = 0; }

#ifdef __cplusplus
extern "C" {
#endif

void* sCalloc(size_t size,size_t count);
void* sMalloc(size_t size);
void* sRealloc(void* old_ptr,size_t size);
void* sRecalloc(void* old_ptr,size_t size);
void sFree(void* ptr);
void sFree2(void** ptr);
size_t sSizeof(void* ptr);
size_t sGetAllocatedMem(void);

#ifdef __cplusplus
}
#endif

#endif