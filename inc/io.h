#ifndef IO_H
#define IO_H

#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

void readf(void* data_ptr,uint32_t size, uint32_t count, FILE* file_ptr);
size_t sizef(FILE* file_ptr);

#endif