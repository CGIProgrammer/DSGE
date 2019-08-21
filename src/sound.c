/*
 ============================================================================
 Name        : sound.c
 Author      : Ivan
 Version     :
 Copyright   : Your copyright notice
 Description : Hello World in C, Ansi-style
 ============================================================================
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "sound.h"
#include "engine.h"
#include "linalg.h"
#include "wav.h"

#include <AL/al.h>
#include <AL/alc.h>
//#include <AL/alut.h>

#define sSoundGetError(err,...) _GetError(__LINE__,err,__VA_ARGS__)

static ALCcontext *Context = 0;
static ALCdevice *Device = 0;
static int sound_err;

typedef struct
{
	uint32_t hash;
	uint32_t index;
} sSoundBuffer;

typedef struct
{
	uint32_t hash;
	uint32_t index;
	sObject *object;
} sSoundSource;

static sSoundBuffer *_sound_buffers = 0;
static sSoundSource *_sound_sources = 0;
static uint32_t _sound_buffers_count = 0;
static uint32_t _sound_sources_count = 0;

static int _GetError(int line,char *error_string,...)
{
	sound_err = alGetError();
    va_list args;
    va_start(args,error_string);
    
    if (sound_err!=AL_NO_ERROR)
    {
        switch (sound_err)
        {
            case AL_INVALID_NAME 		: fprintf(stderr,"sound.c:%d error: invalid name. ",line);break;
            case AL_OUT_OF_MEMORY 		: fprintf(stderr,"sound.c:%d error: out of memory. ",line);break;
            case AL_INVALID_OPERATION 	: fprintf(stderr,"sound.c:%d error: invalid operation. ",line);break;
            case AL_INVALID_VALUE 		: fprintf(stderr,"sound.c:%d error: invalid value. ",line);break;
            case AL_INVALID_ENUM 		: fprintf(stderr,"sound.c:%d error: invalid enum. ",line);break;
        }
        vfprintf(stderr,error_string,args);
    }
    va_end(args);
    return sound_err;
}

int sSoundLoad(char *name)
{
	if (!Context) return SOUND_NO_CONTEXT;
	_sound_buffers = sRealloc(_sound_buffers, sizeof(sSoundBuffer[_sound_buffers_count+1]));
	_sound_buffers[_sound_buffers_count].hash = S_Name2hash(name);

    WAV sound;
    int oresult = WAV_Open(&sound,name);
    if (oresult) return oresult;

    int format = 0x1100 | (sound.bits_per_sample/sound.channels==16) | ((sound.channels==2)<<1);

    alGenBuffers(1, &_sound_buffers[_sound_buffers_count].index);
    oresult = sSoundGetError("sSoundLoad generate buffer\n",1);
	if (oresult) return oresult;
	oresult = sSoundGetError("alGenBuffers %s\n",name);
	if (oresult) return oresult;

	alBufferData(_sound_buffers[_sound_buffers_count].index, format, sound.data, sound.DATA_size, sound.sample_rate);
    oresult = sSoundGetError("sSoundLoad bind data\n",1);
    if (oresult) return oresult;

    WAV_Close(&sound);
    _sound_buffers_count++;
    return SOUND_OK;
}

int sSoundAttachToObject(void *obj,char *sound_name)
{
	if (!Context) return SOUND_NO_CONTEXT;
	sObject *object = obj;
	_sound_sources = sRealloc(_sound_sources,sizeof(sSoundSource[_sound_sources_count+1]));
	alGenSources(1, &_sound_sources[_sound_sources_count].index);
    sSoundGetError("Generate source\n",1);
	if (sound_err!=AL_NO_ERROR)
		return sound_err;

	uint32_t hash = S_Name2hash(sound_name);

	for (uint32_t i=0;i<_sound_buffers_count;i++)
	{
		if (_sound_buffers[i].hash == hash)
		{
			alSourcei(_sound_sources[_sound_sources_count].index, AL_BUFFER,_sound_buffers[i].index);
		    sSoundGetError("Set buffer to source\n",1);
			_sound_sources[_sound_sources_count].object = object;
			break;
		}
		if (i==_sound_buffers_count-1)
		{
			return SOUND_DOES_NOT_EXISTS;
		}
	}

	alSourcei(_sound_sources[_sound_sources_count].index, AL_LOOPING, AL_FALSE);
    sSoundGetError("Set not looping\n",1);
	alSourcePlay(_sound_sources[_sound_sources_count].index);
    sSoundGetError("Playing sound\n",1);
	_sound_sources_count++;
	return SOUND_OK;
}

int sSoundInit(void)
{
	int result;
    Device = alcOpenDevice(NULL);
    if (Device == NULL)
    {
        fprintf(stderr,"Failed to open OAL Device\n");
        return SOUND_NO_DEVICE;
    }
    Context=alcCreateContext(Device,NULL);
    if (Context == NULL)
    {
        fprintf(stderr,"Failed to open OAL Context\n");
        return SOUND_NO_CONTEXT;
    }
    alcMakeContextCurrent(Context);
    result = alcGetError(Device);
    switch (alcGetError(Device))
    {
    case ALC_INVALID_DEVICE		: fprintf(stderr,"ALC_INVALID_Device\n");  break;
    case ALC_INVALID_CONTEXT	: fprintf(stderr,"ALC_INVALID_Context\n"); break;
    case ALC_INVALID_ENUM		: fprintf(stderr,"ALC_INVALID_ENUM\n");    break;
    case ALC_INVALID_VALUE		: fprintf(stderr,"ALC_INVALID_VALUE\n");   break;
    case ALC_OUT_OF_MEMORY		: fprintf(stderr,"ALC_OUT_OF_MEMORY\n");   break;
    }
    return result;
}

void sSoundPlaceSources(void)
{
	if (!Context) return;
	if (_sound_sources_count)
	{
		laType listener = ((sScene*)_sound_sources[0].object->scene)->camera.transform_global;
		float listener_orientation[] = {-listener.a[2], -listener.a[6], -listener.a[10],
				 	 	 	 	 	 	listener.a[1], listener.a[5], listener.a[9]};
		alListener3f(AL_POSITION, listener.a[3], listener.a[7], listener.a[11]);
		sSoundGetError("Set listener position\n",1);
		alListenerfv(AL_ORIENTATION, listener_orientation);
		sSoundGetError("Set listener orientation\n",1);
	}
	for (uint32_t i=0;i<_sound_sources_count;i++)
	{
		int alState;
		//printf("Get source state\n",1);
		alGetSourcei(_sound_sources[i].index, AL_SOURCE_STATE, &alState);
		if (alState==AL_STOPPED)
		{
			//printf("Deleting source %s\n",_sound_sources[i].object->name);
			alDeleteSources(1,&_sound_sources[i].index);
			//printf("Deleting source\n",1);
			memmove(_sound_sources + i, _sound_sources + i+1, sizeof(sSoundSource[_sound_sources_count-i-1]));
			i--;
			_sound_sources_count--;
			continue;
		}
		float position[] = {_sound_sources[i].object->transform_global.a[3],
							_sound_sources[i].object->transform_global.a[7],
							_sound_sources[i].object->transform_global.a[11]};
		float velocity[] = {_sound_sources[i].object->transform_global.a[3]  - _sound_sources[i].object->transform_global_previous.a[3],
							_sound_sources[i].object->transform_global.a[7]  - _sound_sources[i].object->transform_global_previous.a[7],
							_sound_sources[i].object->transform_global.a[11] - _sound_sources[i].object->transform_global_previous.a[11]};

    	alSourcefv(_sound_sources[i].index, AL_POSITION, position);
    	alSourcefv(_sound_sources[i].index, AL_VELOCITY, velocity);
	}
}


void sSoundCloseDevice(void)
{
	if (!Context) return;
	if (_sound_sources)
	{
		for (uint32_t i=0;i<_sound_sources_count;i++)
		{
			alDeleteSources(1, &_sound_sources[i].index);
		}
		sFree(_sound_sources);
	}
	if (_sound_buffers)
	{
		//printf("_sound_buffers, deleting %d\n",_sound_sources_count);
		for (uint32_t i=0;i<_sound_buffers_count;i++)
		{
			alDeleteBuffers(1, &_sound_buffers[i].index);
		}
		sFree(_sound_buffers);
	}
	Context = alcGetCurrentContext();
	Device  = alcGetContextsDevice(Context);
	alcMakeContextCurrent(NULL);
	alcDestroyContext(Context);
	alcCloseDevice(Device);
}
