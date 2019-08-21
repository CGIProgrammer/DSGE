/*
 * sound.h
 *
 *  Created on: 19 июл. 2018 г.
 *      Author: ivan
 */

#ifndef SOUND_H_
#define SOUND_H_

#define SOUND_NO_DEVICE -2
#define SOUND_NO_CONTEXT -1
#define SOUND_OK 0
#define SOUND_OAL_ERROR 1
#define SOUND_FILE_NOT_FOUND 2
#define SOUND_DOES_NOT_EXISTS 3

int sSoundInit(void);
int sSoundLoad(char *name);
int sSoundAttachToObject(void *object,char *sound_name);
void sSoundPlaceSources(void);
void sSoundCloseDevice(void);

#endif /* SOUND_H_ */
