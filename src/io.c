#include "io.h"
#include <stdlib.h>

void readf(void* data_ptr,uint32_t size, uint32_t count, FILE* file_ptr)
{
	if (fread(data_ptr,size,count,file_ptr)<count)
	{
		printf("Error: Unexpected end of file\n");
		exit(-1);
	}
}

size_t sizef(FILE* file_ptr)
{
    size_t place = ftell(file_ptr);
    fseek(file_ptr, 0, SEEK_END);
    size_t fsize = ftell(file_ptr);
    fseek(file_ptr, place, SEEK_SET);
    return fsize;
}