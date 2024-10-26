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

#define GAMMA_CORRECTION (1.0f)
#define GAMMA_CORRECTION_V3 vec3(GAMMA_CORRECTION)

vec3 encodePalYuv(vec3 rgb)
{
    rgb = pow(rgb, GAMMA_CORRECTION_V3); // gamma correction
    return rgb;
    return vec3(
        dot(rgb, vec3(0.299, 0.587, 0.114)),
        dot(rgb, vec3(-0.14713, -0.28886, 0.436)),
        dot(rgb, vec3(0.615, -0.51499, -0.10001))
    );
}

vec3 decodePalYuv(vec3 yuv)
{
    return pow(yuv, 1.0 / GAMMA_CORRECTION_V3);
    vec3 rgb = vec3(
        dot(yuv, vec3(1., 0., 1.13983)),
        dot(yuv, vec3(1., -0.39465, -0.58060)),
        dot(yuv, vec3(1., 2.03211, 0.))
    );
    return pow(rgb, 1.0 / GAMMA_CORRECTION_V3); // gamma correction
}

vec4 fetch_super_resolution(ivec2 pc) {
    camera_zrange = depth_range(camera.projection);
    vec2 half_res = textureSize(lowres_in, 0);
    vec2 full_res = textureSize(hires_in, 0);
    vec2 pixelSize = 1.0 / half_res;
    vec2 pixelCoord = vec2(pc) + vec2(0.5);
    vec2 fragCoord = pixelCoord / textureSize(hires_in, 0);

    ivec2 offset=ivec2(0), lowres_pc = pc / 2;
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
    return mix(prev, curr, temporal_mask);
}
#ifdef SUPER_RESOLUTION
void main() {
    hires_out = fetch_super_resolution(ivec2(pixelCoord.xy));
}
#else
void main()
{
    ivec2 pc = ivec2(pixelCoord.xy);
    ivec2 lowres_pc = ivec2(fragCoord * textureSize(lowres_in, 0));

    vec3 current_vectors = texelFetch(gVectors, lowres_pc, 0).xyz;
    vec2 cfragCoord = fragCoord.xy;
    vec2 pfragCoord = cfragCoord - current_vectors.xy;

    vec4 lastColor = texture(hires_in, pfragCoord, 0);
    
    vec3 antialiased = lastColor.xyz;
    float mixRate = min(lastColor.w, 0.5);
    
    vec3 in0 = texelFetch(lowres_in, lowres_pc, 0).xyz;
    
    antialiased = mix(
        pow(antialiased, GAMMA_CORRECTION_V3),
        pow(in0, GAMMA_CORRECTION_V3),
        mixRate
    );
    antialiased = pow(antialiased, 1.0 / GAMMA_CORRECTION_V3);
    
    vec3 in1 = texelFetch(lowres_in, lowres_pc + ivec2(+1,  0), 0).xyz;
    vec3 in2 = texelFetch(lowres_in, lowres_pc + ivec2(-1,  0), 0).xyz;
    vec3 in3 = texelFetch(lowres_in, lowres_pc + ivec2( 0, +1), 0).xyz;
    vec3 in4 = texelFetch(lowres_in, lowres_pc + ivec2( 0, -1), 0).xyz;
    vec3 in5 = texelFetch(lowres_in, lowres_pc + ivec2(+1, +1), 0).xyz;
    vec3 in6 = texelFetch(lowres_in, lowres_pc + ivec2(-1, +1), 0).xyz;
    vec3 in7 = texelFetch(lowres_in, lowres_pc + ivec2(+1, -1), 0).xyz;
    vec3 in8 = texelFetch(lowres_in, lowres_pc + ivec2(-1, -1), 0).xyz;
    
    antialiased = encodePalYuv(antialiased);
    in0 = encodePalYuv(in0);
    in1 = encodePalYuv(in1);
    in2 = encodePalYuv(in2);
    in3 = encodePalYuv(in3);
    in4 = encodePalYuv(in4);
    in5 = encodePalYuv(in5);
    in6 = encodePalYuv(in6);
    in7 = encodePalYuv(in7);
    in8 = encodePalYuv(in8);
    
    vec3 minColor = min(min(min(in0, in1), min(in2, in3)), in4);
    vec3 maxColor = max(max(max(in0, in1), max(in2, in3)), in4);
    minColor = mix(minColor,
       min(min(min(in5, in6), min(in7, in8)), minColor), 0.5);
    maxColor = mix(maxColor,
       max(max(max(in5, in6), max(in7, in8)), maxColor), 0.5);
    
   	vec3 preclamping = antialiased;
    antialiased = clamp(antialiased, minColor, maxColor);
    
    mixRate = 1.0 / (1.0 / mixRate + 1.0);
    
    vec3 diff = antialiased - preclamping;
    float clampAmount = dot(diff, diff);
    
    mixRate += clampAmount * 4.0;
    mixRate = clamp(mixRate, 0.05, 0.5);
    
    antialiased = decodePalYuv(antialiased);
    
    // antialiased = mix(lastColor.xyz, antialiased.xyz, checker_mask);
    hires_out = vec4(antialiased, mixRate);
}
#endif
