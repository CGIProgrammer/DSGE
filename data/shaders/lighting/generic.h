#ifndef GENERIC_LIGHT_H
#define GENERIC_LIGHT_H
GenericLightUniform
{
    mat4 projection_inv;
    vec4 color;
    vec4 location;
    float power;
    float z_near;
    float distance;
    int shadowmap_index;
}
#endif