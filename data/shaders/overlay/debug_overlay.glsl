#include "../depth_packing.h"
#include "../lighting/shadowmap.h"
#include "debug_gui.h"
#include "../lighting/point.h"
#include "../lighting/spotlight.h"
#extension GL_EXT_debug_printf : enable

float print(SpotLight light, ivec2 offset) {
    float result = 0.0;
    result += print(
        light.color,
        offset
    );
    result += print(
        vec2(light.znear, light.zfar),
        offset + ivec2(0, char_size.y * 1));
    result += print(
        light.angle,
        offset + ivec2(0, char_size.y * 2));
    result += print(
        light.inner_angle,
        offset + ivec2(0, char_size.y * 3));
    return result;
}

void main() {
    int index = 1;
    SpotLight unpacked_light = unpack_spotlight(lights_data, index);
    SpotLight uniform_light;
    uniform_light.znear = ppSpotlights[index].location_znear.w;
    uniform_light.zfar = ppSpotlights[index].direction_zfar.w;
    uniform_light.color = ppSpotlights[index].color.rgb;
    uniform_light.projection_inv = ppSpotlights[index].projection_inv;
    uniform_light.angle = ppSpotlights[index].angle;
    uniform_light.inner_angle = ppSpotlights[index].inner_angle;
    uniform_light.shadow_buffer = ppSpotlights[index].shadow_buffer != 0;
    uniform_light.shadowmap_index = ppSpotlights[index].shadowmap_index;
    ivec2 pc = ivec2(pixelCoord);
    vec4 input_image = texelFetch(image_in, pc, 0);
    debug_gui_resolution = textureSize(image_in, 0);
    debug_gui_coords = pc;
    debug_gui_coords.y = debug_gui_resolution.y - pc.y;
    vec4 font_tex = texelFetch(font, pc, 0);
    float num = print(unpacked_light, ivec2(100));
    num += print(uniform_light, ivec2(100 + 300, 100));
    
    draw_cross(image_out, vec3(1.0));
    image_out = mix(input_image, vec4(1.0), num);
    //image_out = mix(image_out, font_tex, font_tex.a);
}