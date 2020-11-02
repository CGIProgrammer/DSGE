#include "structures/dict.h"
#include "structures/list.h"
#include "memmanager.h"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/*static uint32_t calc_hash(void* item, size_t size) {
    uint32_t result = 0;
    size >>= 2;
    for (size_t i=0; i<size; i++) {
        result ^= *(uint32_t*)((uintptr_t)item + i);
    }
    return result;
}*/

static uint32_t calc_hash(void* item, size_t size)
{
    uint32_t result=0;
	uint64_t a;
	for (size_t i=0; i<size; i++)
	{
		a = result;
		a += (a<<5)^((unsigned char*)item)[i];
		result = a ^ (a>>17);
	}
	return result;
}

size_t sDictGetTableSize(sDict* dict)
{
    return sSizeof(dict->cells) / sizeof(void*);
}

sDict sDictNew(size_t table_size)
{
    sDict result = {
        sNewArray(sHashTableCell*, table_size),
        0
    };
    return result;
}

void sDictRealloc(sDict* dict, size_t table_size)
{
    sDict new_dict = sDictNew(table_size);
    size_t old_table_size = sDictGetTableSize(dict);
    sHashTableCell cell;
    for (size_t i=0; i<old_table_size; i++)
    {
        if (dict->cells[i]) {
            size_t s = sListGetSize(dict->cells[i]);
            for (size_t j=0; j<s; j++) {
                cell = dict->cells[i][j];
                size_t ia = cell.key_hash % table_size;
                sListPushBack(new_dict.cells[ia], cell);
            }
        }
    }
    new_dict.pairs_count = dict->pairs_count;
    for (size_t i=0; i<old_table_size; i++) {
        sDelete(dict->cells[i]);
    }
    sDelete(dict->cells);
    *dict = new_dict;
}

uint8_t sDictHaveItem(sDict* dict, void* key, size_t key_size, size_t* index_a, size_t* index_b, uint32_t* hash_out)
{
    size_t ts = sDictGetTableSize(dict);
    if (!ts) return 0;

    uint32_t hash = calc_hash(key, key_size);
    size_t ia = hash % ts;
    sHashTableCell* cells = dict->cells[ia];
    int64_t lin_index = -1; //listGetIndexOf(&keys_arr, key, key_size);
    size_t n = sListGetSize(cells);
    
    if (hash_out) *hash_out = hash;

    for (size_t i=0; i<n; i++)
    {
        if (cells[i].key_hash==hash) {
            if (!memcmp((const void*)cells[i].key, (const void*)key, key_size)) {
                lin_index = i;
                break;
            }
        }
    }
    
    //printf("sDictHaveItem(sDict* %p, void* %s, size_t %lu) : list_size %lu; lin_index %lu\n", dict, key, key_size, n, lin_index);
    if (index_a) *index_a = ia;
    if (index_b) *index_b = lin_index;

    return lin_index>=0;
}

void* sDictGetItem(sDict* dict, void* key, size_t key_size)
{
    size_t index_a, index_b;
    uint8_t hi = sDictHaveItem(dict, key, key_size, &index_a, &index_b, 0);
    if (hi) {
        return dict->cells[index_a][index_b].value;
    }
    else {
        return 0;
    }
}

void* sDictGetItemKW(sDict* dict, char* key)
{
    size_t key_size = strlen(key);
    return sDictGetItem(dict, key, key_size);
}

uint8_t sDictHaveItemKW(sDict* dict, char* key)
{
    return sDictHaveItem(dict, key, strlen(key), 0, 0, 0);
}

void sDictAddItem(sDict* dict, void* key, size_t key_size, void* item)
{
    size_t index_a=MAX_INDEX, index_b=MAX_INDEX;
    size_t table_size = sDictGetTableSize(dict);
    if (dict->pairs_count >= table_size) {
        if (table_size==0) {
            sDictRealloc(dict, 32);
        } else {
            sDictRealloc(dict, table_size * 2);
        }
    }
    uint32_t hash=0;
    uint8_t hi = sDictHaveItem(dict, key, key_size, &index_a, &index_b, &hash);
    if (hi) {
        void* new_key = sRealloc(dict->cells[index_a][index_b].key, key_size);
        memcpy(new_key, key, key_size);
        dict->cells[index_a][index_b].key   = new_key;
        dict->cells[index_a][index_b].value = item;
    } else {
        void* new_key = sNewArray(char, key_size);
        sHashTableCell new_cell = {hash, new_key, item};
        memcpy(new_key, key, key_size);
        sListPushBack(dict->cells[index_a], new_cell);
        dict->pairs_count++;
    }
}

void sDictAddItemKW(sDict* dict, char* key, void* item)
{
    size_t key_size = strlen(key);
    sDictAddItem(dict, key, key_size, item);
}

void sDictRemoveItem(sDict* dict, void* key, size_t key_size)
{
    size_t index_a, index_b;
    uint8_t hi = sDictHaveItem(dict, key, key_size, &index_a, &index_b, 0);
    if (!hi) return;
    sDelete(dict->cells[index_a][index_b].key)
    sListPopIndex(dict->cells[index_a], index_b);
    dict->pairs_count--;
    size_t table_size = sDictGetTableSize(dict);
    if (dict->pairs_count < table_size * 0.4 && (table_size>>2)>=32) {
        sDictRealloc(dict, table_size>>2);
    }
}

void sDictRemoveItemByValue(sDict* dict, void* key)
{
    size_t ts = sDictGetTableSize(dict);
    for (size_t i=0; i<ts; i++)
    {
        if (!dict->cells[i]) continue;
        size_t ls = sListGetSize(dict->cells[i]);
        for (size_t j=0; j<ls; j++)
        {
            if (dict->cells[i][j].value == key)
            {
                sDelete(dict->cells[i][j].key);
                sListPopIndex(dict->cells[i], j);
                dict->pairs_count--;
                return;
            }
        }
    }
}

void sDictRemoveItemKW(sDict* dict, char* key)
{
    size_t key_size = strlen(key);
    return sDictRemoveItem(dict, key, key_size);
}

void sDictDelete(sDict* dict)
{
    if (!dict->cells) return;
    size_t table_size = sDictGetTableSize(dict);
    for (size_t i=0; i<table_size; i++)
    {
        if (dict->cells[i]) {
            size_t s = sListGetSize(dict->cells[i]);
            for (size_t j=0; j<s; j++) {
                sDelete(dict->cells[i][j].key);
            }
            sDelete(dict->cells[i]);
        }
    }
    sDelete(dict->cells);
    dict->pairs_count = 0;
}