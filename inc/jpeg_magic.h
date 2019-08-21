/*
 * jpeg_magic.h
 *
 *  Created on: 23 авг. 2017 г.
 *      Author: ivan
 */

#ifndef JPEG_MAGIC_H_
#define JPEG_MAGIC_H_

#include <stdio.h>
#include <jpeglib.h>
#include <setjmp.h>
#include <stdint.h>
#include <string.h>

int read_JPEG_file (char * filename, void** ptr, uint16_t* width, uint16_t* height);
void write_JPEG_file (char * filename,JSAMPLE* img_buffer,uint16_t img_width,uint16_t img_height, uint8_t format, int quality);
void write_JPEG_stream (FILE * stream,JSAMPLE* img_buffer,uint16_t img_width,uint16_t img_height, uint8_t format, int quality);

#endif /* JPEG_MAGIC_H_ */
