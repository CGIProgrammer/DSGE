/*
 * locales.h
 *
 *  Created on: 7 янв. 2018 г.
 *      Author: ivan
 */

#ifndef LOCALES_H_
#define LOCALES_H_

#include <stdio.h>

#ifdef RUSSIAN

#define GLFW_START_ERROR {fprintf(stderr,"Ошибка: не удалось создать окно.\n");exit(-1);}
#define GL_START_ERROR {fprintf(stderr,"Ошибка: не удалось инициализировать OpenGL. У вас установлены драйверы на видеокарту?\n");exit(-1);}

#endif
#ifndef RUSSIAN

#define GLFW_START_ERROR {fprintf(stderr,"Error: failed to create window.\n");exit(-1);}
#define GL_START_ERROR {fprintf(stderr,"Error: failed to start OpenGL. Do you have driver for your graphgics card?\n");exit(-1);}

#endif

#endif /* LOCALES_H_ */
