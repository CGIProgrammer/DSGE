/*
 * image.h
 *
 *  Created on: 23 авг. 2017 г.
 *      Author: ivan
 */

#ifndef IMAGE_H_
#define IMAGE_H_

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include "jpeg_magic.h"

#define clamp(a,b,val) ((val)<(a) ? (a) : ((val)>(b) ? (b) : (val)))
#define side 8

typedef uint64_t size_a;

typedef enum
{
	RGBI = 0,
	YBRI,
	GRAYI
} PIX_FORMAT;

typedef struct
{
	uint8_t* levels;
	uint8_t* data;
	uint16_t width;
	uint16_t height;
	PIX_FORMAT format;
	size_a size;
	uint32_t levels_count;
} IMAGE;

void IMAGE_New(IMAGE* img,uint16_t width,uint16_t height,PIX_FORMAT format);
void IMAGE_Load_JPEG(IMAGE* img,char* filename);
void IMAGE_Load_STUP(IMAGE* img,char* filename);
void IMAGE_Load_STUP2(IMAGE* value,IMAGE* cb,IMAGE* cr,char* filename);
void IMAGE_Save_JPEG(IMAGE* img,char* name);
void IMAGE_Stream_JPEG(IMAGE* img,FILE* stream);
void IMAGE_Free(IMAGE* img);
void IMAGE_Merge(IMAGE* img,IMAGE* ch1,IMAGE* ch2,IMAGE* ch3);
void IMAGE_Save_STUP(IMAGE* img,char* filename);
void IMAGE_Copy(IMAGE* img,IMAGE* new_img);
IMAGE* IMAGE_Split(IMAGE* img);
void IMAGE_Scale(IMAGE* img,_Bool scale);
void IMAGE_Scaled(IMAGE* img,IMAGE* img2,_Bool scale);
void IMAGE_Convert(IMAGE* img,PIX_FORMAT format);
void IMAGE_CB(IMAGE* img,float contrast,int16_t bright);
void IMAGE_Edge_Filter(IMAGE* img);
void IMAGE_Blur(IMAGE* img,uint8_t radius);
void IMAGE_Invert(IMAGE* img);
void IMAGE_Selective_Blur(IMAGE*,float,float,float);
void IMAGE_Threshold(IMAGE* img,int8_t val);
void IMAGE_Overlay(IMAGE* img,IMAGE* img2);
uint8_t _get_2bit(uint8_t* arr,size_a index);
uint8_t _get_4bit(uint8_t* arr,size_a index);

#endif /* IMAGE_H_ */
