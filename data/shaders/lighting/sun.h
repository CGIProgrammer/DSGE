#ifndef SUN_LIGHT_H
#define SUN_LIGHT_H
#include generic.h
SunLightUniform
{
    GenericLightUniform base;
    vec4 direction;
    float size;
    float reserved1;
    float reserved2;
    float reserved3;
}
#endif