#include "constants.h"

vec2 dither[4] = vec2[](
    vec2(-0.5, -0.5) / vec2(resolution.dimensions),
    vec2(-0.5,  0.5) / vec2(resolution.dimensions),
    vec2( 0.5, -0.5) / vec2(resolution.dimensions),
    vec2( 0.5,  0.5) / vec2(resolution.dimensions)
);

vec2 interpolate_random_angles(uint frame, const uint coefficient) {
    float flfr = float(frame) / coefficient;
    uint fr = int(flfr);
    return mix(random_angles[fr & 0xF], random_angles[(fr+1) & 0xF], fract(flfr));
}

void main()
{
    #ifdef SHADOWMAP
    mat3 TBN;
    vec4 position, position_prev;
    vec3 view_vector;
    #endif
    texture_uv = vec2(v_tex1.x, 1.0 - v_tex1.y);
    mat4 model = camera.transform_inverted * transform;
    position = camera.projection * model * vec4(v_pos, 1.0);

    #ifdef SUPER_RESOLUTION
    vec2 dither_angle = dither[timer.frame & 0x3];
    vec2 dither_angle_prev = dither[timer.frame & 0x3];
    #else
    vec2 dither_angle = random_angles[timer.frame & 0xF] / vec2(resolution.dimensions);
    vec2 dither_angle_prev = random_angles[(timer.frame) & 0xF] / vec2(resolution.dimensions);
    #endif

    #ifndef SHADOWMAP
    position_prev = camera.projection * camera.transform_prev_inverted * transform_prev * vec4(v_pos, 1.0);
    float nLength = length(v_nor);
    TBN = mat3(v_tan, -v_bin, v_nor);
    TBN = mat3(transform[0].xyz, transform[1].xyz, transform[2].xyz) * TBN;
    #endif
    triangle_index = gl_InstanceIndex;
    world_position = (model * vec4(v_pos, 1.0)).xyz;
    view_vector = (camera.transform_inverted * position).xyz;
    #ifdef SHADOWMAP
    //position.xy -= random_angles[timer.frame & 0xF] / vec2(resolution.dimensions) * position.w;
    #else
    position.xy -= dither_angle * position.w;
    position_prev.xy -= dither_angle_prev * position.w;
    #endif
    gl_Position = position;
}