/*
 * wav.c
 *
 *  Created on: 14 авг. 2017 г.
 *      Author: ivan
 */
#include "wav.h"
#include "engine.h"

void WAV_From_Data(WAV* snd,char* data,uint32_t samples,uint32_t sample_rate,uint16_t channels,uint16_t precision)
{
	snd->DATA_size = channels*precision*samples;
	memcpy(snd->RIFF,"RIFF",4);
	snd->chunk_size = snd->DATA_size+36;
	memcpy(snd->WAVE,"WAVE",4);
	memcpy(snd->FMT,"fmt ",4);
	snd->subchunk_size = 16;
	snd->audio_format = 1;
	snd->channels = channels;
	snd->sample_rate = sample_rate;
	snd->byte_rate = sample_rate*precision;
	snd->block_align = precision*channels;
	snd->bits_per_sample = precision<<3;
	memcpy(snd->DATA,"data",4);
	snd->data = data;
	snd->samples = samples;
	snd->opened=1;
}

int WAV_Open(WAV* sound,char* name)
{
	sound->opened = 0;
	uint32_t read_bytes = 0;
	FILE* file = fopen(name,"rb");
	if (!file)
	{
		return 1;
	}
	read_bytes = fread(sound,44,1,file);
	uint8_t trys=0;
	while (!(sound->DATA[0]=='d'&&
			 sound->DATA[1]=='a'&&
			 sound->DATA[2]=='t'&&
			 sound->DATA[3]=='a') && trys++<5)
	{
		fseek(file, sound->DATA_size, SEEK_CUR);
		read_bytes += fread(&(sound->DATA),8,1,file);
	}
	sound->samples = sound->DATA_size/sound->block_align;
	sound->data = sMalloc(sound->DATA_size);
	read_bytes = fread(sound->data,sound->DATA_size,1,file);
	fclose(file);
	sound->opened=1;
	return 0;
}

void WAV_Smooth(WAV* snd)
{
	for (uint32_t i=16;i<snd->samples*snd->channels-1;i+=8)
	{
		((uint8_t*)snd->data)[i] = (((uint8_t*)snd->data)[i-2] + ((uint8_t*)snd->data)[i+2])/2;
		((uint8_t*)snd->data)[i+1] = (((uint8_t*)snd->data)[i-1] + ((uint8_t*)snd->data)[i+3])/2;
	}
}

void WAV_New(WAV* snd,uint32_t samples,uint32_t sample_rate,uint16_t channels,uint16_t precision)
{
	snd->DATA_size = channels*precision*samples;
	memcpy(snd->RIFF,"RIFF",4);
	snd->chunk_size = snd->DATA_size+36;
	memcpy(snd->WAVE,"WAVE",4);
	memcpy(snd->FMT,"fmt ",4);
	snd->subchunk_size = 16;
	snd->audio_format = 1;
	snd->channels = channels;
	snd->sample_rate = sample_rate;
	snd->byte_rate = sample_rate*precision;
	snd->block_align = precision*channels;
	snd->bits_per_sample = precision<<3;
	memcpy(snd->DATA,"data",4);
	snd->data = sCalloc(samples,channels*precision);
	snd->samples = samples;
	snd->opened=1;
}

void WAV_Convert_to_UINT8M(WAV* sound)
{
	/*if (sound->audio_format == 3)
		printf("%d bit float, channels %d\n",sound->bits_per_sample,sound->channels);
	if (sound->audio_format == 1)
		printf("%d bit int, channels %d\n",sound->bits_per_sample,sound->channels);
	*/
	if (!sound->opened) return;
	if (sound->audio_format == 3)
	{
		if (sound->bits_per_sample==32)
			for (uint32_t i=0;i<sound->samples;i++)
			{
				float sample = 0.0;
				for (uint32_t c=0;c<sound->channels;c++)
				{
					sample += ((float*)sound->data)[i*sound->channels+c]/sound->channels;
				}
				((uint8_t*)sound->data)[i] = (sample*0.5+0.5)*255+0.5;
			}
		else if (sound->bits_per_sample==64)
			for (uint32_t i=0;i<sound->samples;i++)
			{
				double sample = 0.0;
				for (uint8_t c=0;c<sound->channels;c++)
					sample += ((double*)sound->data)[i*sound->channels+c]/sound->channels;
				((uint8_t*)sound->data)[i] = (sample*0.5+0.5)*255+0.5;
			}
	}
	else if (sound->audio_format == 1)
	{
		if (sound->bits_per_sample==16)
			for (uint32_t i=0;i<sound->samples;i++)
			{
				int32_t sample = 0;
				for (uint8_t c=0;c<sound->channels;c++)
					sample += ((int16_t*)sound->data)[i*sound->channels+c];
				sample /= sound->channels;
				((uint8_t*)sound->data)[i] = (sample+(1<<15))>>8;
			}
		else if (sound->bits_per_sample==24)
			for (uint32_t i=0;i<sound->samples;i++)
			{
				int32_t sample = 0;
				for (uint8_t c=0;c<sound->channels;c++)
					sample += int24(sound->data,i*sound->channels);
				sample /= sound->channels;
				((uint8_t*)sound->data)[i] = (sample+(1<<23))>>16;
			}
		else if (sound->bits_per_sample==32)
			for (uint32_t i=0;i<sound->samples;i++)
			{
				int64_t sample = 0;
				for (uint8_t c=0;c<sound->channels;c++)
					sample += ((int32_t*)sound->data)[i*sound->channels+c];
				sample /= sound->channels;
				((uint8_t*)sound->data)[i] = (sample+(1<<31))>>24;
			}
	}
	sRealloc(sound->data,sound->samples);
	sound->bits_per_sample = 8;
	sound->byte_rate = sound->sample_rate;
	sound->audio_format = 1;
	sound->channels = 1;
	sound->block_align = 1;
	sound->DATA_size = sound->samples*sound->channels*(sound->bits_per_sample>>3);
	sound->chunk_size = sound->DATA_size+36;
}

void WAV_Save(WAV* snd, const char* name)
{
	if (snd->opened)
	{
		FILE* file = fopen(name,"wb");
		fwrite(snd,44,1,file);
		fwrite(snd->data,(snd->bits_per_sample>>3)*snd->channels,snd->samples,file);
		fclose(file);
	}
}

int32_t WAV_Max(WAV* snd,uint32_t start,uint32_t length)
{
	int32_t val = 0;
	if (snd->bits_per_sample == 8)
	{
		val = 0;
		while (length--)
		{
			if (((uint8_t*)snd->data)[start]>val) val = ((uint8_t*)snd->data)[start];
			start++;
		}
	}
	else if (snd->bits_per_sample == 16)
		while (length--)
		{
			if (((int16_t*)snd->data)[start]>val) val = ((int16_t*)snd->data)[start];
			start++;
		}
	else if (snd->bits_per_sample == 32)
		while (length--)
		{
			if (((int32_t*)snd->data)[start]>val) val = ((int32_t*)snd->data)[start];
			start++;
		}
	return val;
}

int32_t WAV_Min(WAV* snd,uint32_t start,uint32_t length)
{
	int32_t val = 0;
	if (snd->bits_per_sample == 8)
	{
		val = 255;
		while (length--)
		{
			if (((uint8_t*)snd->data)[start]<val) val = ((uint8_t*)snd->data)[start];
			start++;
		}
	}
	else if (snd->bits_per_sample == 16)
		while (length--)
		{
			if (((int16_t*)snd->data)[start]<val) val = ((int16_t*)snd->data)[start];
			start++;
		}
	else if (snd->bits_per_sample == 32)
		while (length--)
			{
				if (((int32_t*)snd->data)[start]<val) val = ((int32_t*)snd->data)[start];
				start++;
			}
	return val;
}

void WAV_ReLevels(WAV* snd,uint32_t start,uint32_t length,char* levels)
{
	uint32_t max = (*(uint16_t*)levels)>>8;
	uint32_t min = (*(uint16_t*)levels)&0xFF;
	//printf("max=%d, min=%d\n",max,min);
	if (snd->bits_per_sample == 8)
	{
		while (length--)
		{
			((uint8_t*)snd->data)[start] = (float)(((uint8_t*)snd->data)[start])/255.0*fabs((float)(max-min))+min;
			start++;
		}
	}
}

/*void WAV_Levels(WAV* snd,uint32_t start,uint32_t length,char* levels)
{
	uint32_t max = WAV_Max(snd, start, length);
	uint32_t min = WAV_Min(snd, start, length);
	if (max-min<=1)
	{
		*(uint16_t*)levels = 0xFF00;
		return;
	}
	if (snd->bits_per_sample == 8)
	{
		*(uint16_t*)levels = (max<<8)|min;
		while (length--)
		{
			((uint8_t*)snd->data)[start] = (float)(((uint8_t*)snd->data)[start]-min)/(float)(max-min)*255.0;
			start++;
		}
	}

	else if (snd->bits_per_sample == 16)
		while (length--)
		{
			((uint16_t*)snd->data)[start] = (((uint16_t*)snd->data)[start]-min)*(max-min);
			start++;
		}

	else if (snd->bits_per_sample == 32)
		while (length--)
		{
			((uint32_t*)snd->data)[start] = (((uint32_t*)snd->data)[start++]-min)*(max-min);
			start++;
		}
}*/

void WAV_Close(WAV* snd)
{
	if (snd->opened)
	{
		snd->opened=0;
		sFree(snd->data);
	}
}
