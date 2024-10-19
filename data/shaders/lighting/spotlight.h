#ifndef SPOTLIGHT_H
#define SPOTLIGHT_H
#include generic.h
SpotlightUniform
{
    GenericLightUniform base;
    vec4 direction;
    float inner_angle;
    float outer_angle;
    float reserved1;
    float reserved2;
}
#endif