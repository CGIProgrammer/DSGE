#include "../depth_packing.h"
#include "../constants.h"

/// @brief Смещения для квадрата 3x3.
const ivec2 offsets[9] = ivec2[](
    ivec2(0,  0),  ivec2(-1,  1), ivec2(0,  1),
    ivec2(1,  1),  ivec2(-1,  0), ivec2(1,  0),
    ivec2(-1, -1), ivec2(0, -1),  ivec2(1, -1)
);

/// @brief Координаты клеток.
const ivec2 dither[4] = ivec2[](
    ivec2(0, 0),
    ivec2(0, 1),
    ivec2(1, 0),
    ivec2(1, 1)
);

const vec2 dither_angles[4] = vec2[](
    vec2(-0.5, -0.5),
    vec2(-0.5,  0.5),
    vec2( 0.5, -0.5),
    vec2( 0.5,  0.5)
);

vec2 camera_zrange;


/// @brief Вычисляет четвертную маску для отбора пикселей, которые должны быть в указаном кадре.
/// @param pc Целочисленные координаты фрагмента.
/// @param frame Номер кадра.
/// @return 1 если текущий пиксель должен обновиться. В протоивном случае 0.
float checker_mask(ivec2 pc, uint frame) {
    ivec2 checker_offset = (pc & 0x1) - dither[frame & 0x3];
    return float(checker_offset.x==0 && checker_offset.y==0);
}

/// @brief Вычисляет четвертную маску на основе значения из выборки random_angles.
/// Это может быть полезно для добавления временного сглаживания.
/// @param pc Целочисленные координаты фрагмента.
/// @param frame Номер кадра.
/// @return 1 если текущий пиксель должен обновиться. В протоивном случае 0.
float random_checker_mask(ivec2 pc, uint frame) {
    vec2 rnd = random_angles[frame & 0xF];
    ivec2 checker_offset = (pc & 0x1) - ivec2(int(rnd.x>=0), int(rnd.y>=0));
    return float(checker_offset.x==0 && checker_offset.y==0);
}

/// @brief Выборка минимальноого значения в квадрате 3x3.
/// @param smplr Сэмплер для выборки.
/// @param pc Целочисленные координаты фрагмента.
/// @param component Компонента, по которой произойдёт выборка.
/// @param offset Выход для найденного смещения относительно pc.
/// @return Полное значение выборки.
vec4 min_3x3(sampler2D smplr, ivec2 pc, const int component, out ivec2 offset) {
    vec4 result = texelFetch(smplr, pc, 0);
    for (int i=1; i<9; i++) {
        vec4 smpl = texelFetch(smplr, pc+offsets[i], 0);
        if (smpl[component] < result[component]) {
            result = smpl;
            offset = offsets[i];
        }
    }
    return result;
}

/// @brief Выборка минимальноого значения в квадрате 3x3 без сохранения смещения.
/// @param smplr Сэмплер для выборки.
/// @param pc Целочисленные координаты фрагмента.
/// @param component Компонента, по которой произойдёт выборка.
/// @return Полное значение выборки.
vec4 min_3x3(sampler2D smplr, ivec2 pc, const int component) {
    vec4 result = texelFetch(smplr, pc, 0);
    for (int i=1; i<9; i++) {
        vec4 smpl = texelFetch(smplr, pc+offsets[i], 0);
        if (smpl[component] < result[component]) {
            result = smpl;
        }
    }
    return result;
}
/*
/// @brief Выборка минимальноого значения в квадрате 3x3 без сохранения смещения.
/// @param smplr Сэмплер для выборки.
/// @param pc Нормализованные координаты фрагмента.
/// @param component Компонента, по которой произойдёт выборка.
/// @return Полное значение выборки.
vec4 min_3x3(sampler2D smplr, vec2 pc, const int component) {
    vec4 result = texture(smplr, pc, 0);
    vec2 ires = 1.0 / textureSize(smplr, 0);
    for (int i=1; i<9; i++) {
        vec4 smpl = texture(smplr, pc + offsets[i] * ires, 0);
        if (smpl[component] < result[component]) {
            result = smpl;
        }
    }
    return result;
}

/// @brief Выборка минимальноого значения в квадрате 3x3 с сохранением смещения.
/// @param smplr Сэмплер для выборки.
/// @param pc Нормализованные координаты фрагмента.
/// @param offset Выход для найденного смещения относительно pc.
/// @param component Компонента, по которой произойдёт выборка.
/// @return Полное значение выборки.
vec4 min_3x3(sampler2D smplr, vec2 pc, const int component, out ivec2 offset) {
    vec4 result = texture(smplr, pc, 0);
    vec2 ires = 1.0 / textureSize(smplr, 0);
    for (int i=1; i<9; i++) {
        vec4 smpl = texture(smplr, pc + offsets[i] * ires, 0);
        if (smpl[component] < result[component]) {
            result = smpl;
            offset = offsets[i];
        }
    }
    return result;
}*/

/// @brief Вычисление маски для удаления следа от предыдущих кадров
/// @param pc Целочисленные координаты фрагмента в разрешении рендеринга.
/// @param hires_pc Целочисленные координаты фрагмента в повышенном рендеринга.
/// @param offset Смещение,
/// @return 
float  calc_ghosting_mask(float c_depth, ivec2 hires_pc) {
    float p_depth_1 = texelFetch(hires_in, hires_pc, 0).w;
    float ep_depth_diff = max(0.0, c_depth - p_depth_1); // + max(0.0, p_depth_1 - c_depth) * 0.1;
    return ep_depth_diff;
}

float min3x3_ghosting_mask(float c_depth, ivec2 hires_pc) {
    float result = calc_ghosting_mask(c_depth, hires_pc);
    for (int i=1; i<9; i++) {
        float smpl = calc_ghosting_mask(c_depth, hires_pc + offsets[i]);
        if (smpl < result) {
            result = smpl;
        }
    }
    return result;
}

vec4 denoised_sampler(sampler2D smplr, ivec2 pc) {
    float w = 0.2;
    vec4 result = vec4(0.0);
    float d = texelFetch(gDepth, pc, 0).r;
    int counter = 0;
    for (int i=1; i<9; i++) {
        float dd = texelFetch(gDepth, pc+offsets[i], 0).r;
        if (dd < d) {
            vec4 smpl = texelFetch(smplr, pc+offsets[i], 0);
            float coeff = clamp((d - dd) * 100.0, 0.0, 1.0);
            result += smpl * coeff;
            w += coeff;
            counter += 1;
            if (counter >= 8) {
                return result / w;
            }
        }
    }
    return texelFetch(smplr, pc, 0);
}

void main() {
    camera_zrange = depth_range(camera.projection);
    vec2 half_res = textureSize(lowres_in, 0);
    vec2 full_res = textureSize(hires_in, 0);
    vec2 pixelSize = 1.0 / half_res;

    ivec2 pc = ivec2(pixelCoord.xy);

    ivec2 offset=ivec2(0), lowres_pc = ivec2(fragCoord * textureSize(lowres_in, 0));
    float c_depth;
    c_depth = min_3x3(gDepth, lowres_pc, 0, offset).x;
    c_depth = linearize_depth(c_depth, camera_zrange.x, camera_zrange.y);

    #ifdef SUPER_RESOLUTION
    float checker_mask = checker_mask(pc, timer.frame) * 0.2;
    #else
    float checker_mask = 0.1;
    #endif
    vec3 current_vectors = texelFetch(gVectors, lowres_pc + offset, 0).xyz;
    current_vectors.xy += offset / textureSize(lowres_in, 0);
    vec2 cfragCoord = fragCoord.xy;
    vec2 pfragCoord = cfragCoord - current_vectors.xy;

    vec4 curr = vec4(texelFetch(lowres_in, lowres_pc, 0).rgb, c_depth);

    float ghosting_mask = calc_ghosting_mask(c_depth, pc);
    float border_mask = float(pfragCoord.x <= pixelSize.x || pfragCoord.x >= 1.0-pixelSize.x || pfragCoord.y <= pixelSize.y || pfragCoord.y >= 1.0 - pixelSize.y);
    float motion_amount = length(current_vectors.xy) * 10.0;
    motion_amount = motion_amount / (1.0 + motion_amount);
    
    vec4 prev = texture(hires_in, pfragCoord.xy);/* - 
        sign(current_vectors.xy) * 
        min(
            abs(current_vectors.xy)*0.1,
            1.0/textureSize(hires_in, 0))
        );*/
    
    vec4 temporal_mask = clamp(
        + max(checker_mask.rrrr, 0.0)
        + max(border_mask.rrrr, 0.0)
        + max(ghosting_mask.rrrr, 0.0)
    , motion_amount, 1.0);
    hires_out = mix(prev, curr, temporal_mask);
    //hires_out.w += current_vectors.z;

    /*if (fragCoord.x > 0.5) {
        //hires_out.xyz = clamp(border_mask.rrr + ghosting_mask.rrr, motion_amount, 1.0);
        hires_out.xyz = 1.0 / (c_depth.rrr + 1.0);
    }*/
}
