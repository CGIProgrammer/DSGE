/*
 * wav.h
 *
 *  Created on: 14 авг. 2017 г.
 *      Author: ivan
 */

#ifndef WAV_H_
#define WAV_H_
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>

#define int24(arr,ind) ((*(int32_t*)&(((uint8_t*)(arr))[(ind)*3]))&0xFFFFFF)
#define CFI(fl) ((uint8_t*)&(fl))
#define BE(fl) ((CFI(fl)[0]<<24) | (CFI(fl)[1]<<8) | (CFI(fl)[2]>>8) | (CFI(fl)[3]>>24))

typedef enum
{
    WAV_OK = 0,
    WAV_FMT,
    WAV_RIFF
} WAV_RESULT;

typedef struct
{
    char RIFF[4];
    uint32_t chunk_size;
    char WAVE[4];
    char FMT[4];
    uint32_t subchunk_size;
    uint16_t audio_format;
    uint16_t channels;
    uint32_t sample_rate;
    uint32_t byte_rate;
    uint16_t block_align;
    uint16_t bits_per_sample;
    char DATA[4];
    uint32_t DATA_size;
    uint32_t samples;
    _Bool opened;
    void* data;
} WAV;

typedef struct
{
    union
    {
        char d[3];
        int32_t di;
    };
} int24_t;

typedef struct
{
    char d[3];
} int24_s;

int WAV_Open(WAV* sound,char* name);
void WAV_Convert_to_UINT8M(WAV* sound);
void WAV_Save(WAV* snd, const char* name);
void WAV_New(WAV* snd,uint32_t samples,uint32_t sample_rate,uint16_t channels,uint16_t precision);
void WAV_From_Data(WAV* snd,char* data,uint32_t samples,uint32_t sample_rate,uint16_t channels,uint16_t precision);
void WAV_Smooth(WAV* snd);
int32_t WAV_Max(WAV* snd,uint32_t start,uint32_t length);
int32_t WAV_Min(WAV* snd,uint32_t start,uint32_t length);
//void WAV_Levels(WAV* snd,uint32_t start,uint32_t length,char* levels);
void WAV_Close(WAV* snd);

#endif /* WAV_H_ */
