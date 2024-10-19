#ifndef SHADOWMAP_GLSL
#define SHADOWMAP_GLSL

#include "../random.glsl"
#include "../depth_packing.h"

float shadow_sample(sampler2DArray shadowmap, int index, mat4 proj, vec3 n, vec3 p, vec3 l_norm, float fr_dist, float zfar, float znear, int noise_seed)
{
    vec4 shadowmap_homogeneous = proj * vec4(p, 1.0);
    vec2 shadowmap_2d_coord = shadowmap_homogeneous.xy;
    float shadow = 1.0;
    float sm_dist = 0.0;
    float bias = max(0.05 * (1.0 - dot(n, l_norm)), 0.1);
    float sm_size = 2.0 / textureSize(shadowmap, 0).x;
    int samples = 1;
    if (shadowmap_homogeneous.w > 0.0) {
        shadowmap_2d_coord = (shadowmap_2d_coord / shadowmap_homogeneous.w) * 0.5 + 0.5;
        shadow = 0.0;
        for (int i=0; i<samples; i++) {
            vec2 blue_noise = bluerand4(blue_noise, i+samples*noise_seed).xy * sm_size;
            ivec2 crd = ivec2((shadowmap_2d_coord + blue_noise * 0.5) * textureSize(shadowmap, 0).xy);
            sm_dist = linearize_depth(texelFetch(shadowmap, ivec3(crd, index), 0).r, znear, zfar);
            //sm_dist = linearize_depth(texture(shadowmap, vec3(shadowmap_2d_coord + blue_noise * 0.5, index)).r, znear, zfar);
            shadow += float(fr_dist < sm_dist.r + bias);
        }
        shadow /= samples;
    } else {
        shadow = 1.0;
    }
    return shadow;
}

vec4 subTextureCube(sampler2DArray tex, vec3 direction, int layer)
{
    const float epsilon = 1e-6;
    const float limit = 1.0 - epsilon;
    vec3 lv = direction / max(max(abs(direction.x), abs(direction.y)), abs(direction.z));
    vec3 color = vec3(1);
    vec2 crd = vec2(0);
    int offset = -1;
    // Низ
    if (lv.z < -limit) {
        crd = lv.xy * vec2(-0.5, -0.5) + 0.5;
        offset = 5;
    }
    // Верх
    else if (lv.z > limit) {
        crd = lv.xy * vec2(0.5, -0.5) + 0.5;
        offset = 4;
    }
    // Зад
    else if (lv.y < -limit) {
        crd = lv.xz * vec2(0.5, -0.5) + 0.5;
        offset = 3;
    }
    // Перед
    else if (lv.y > limit) {
        crd = lv.xz * 0.5 + 0.5;
        offset = 2;
    }
    // Лево
    else if (lv.x < -limit) {
        crd = lv.zy * vec2(0.5, -0.5) + 0.5;
        offset = 1;
    }
    // Право
    else if (lv.x > limit) {
        crd = lv.zy * vec2(-0.5, -0.5) + 0.5;
        offset = 0;
    }
    else {
    }
    if (offset >= 0) {
        return texelFetch(tex, ivec3(crd*textureSize(tex, 0).xy, layer*6 + offset), 0);
    } else {
        return vec4(0);
    }
}

float subTextureCubeShadow(sampler2DArray tex, int layer, vec3 source_location, vec3 fragment_location, vec3 fragment_normal, vec3 dir_offset, float z_near, float z_far)
{
    vec2 ts = textureSize(tex, 0).xy;
    const float epsilon = 1e-6;
    const float limit = 1.0 - epsilon;
    vec3 direction = fragment_location - source_location;
    vec3 nd = normalize(normalize(direction) + dir_offset/ts.x*4.0);
    vec3 lv = nd / max(max(abs(nd.x), abs(nd.y)), abs(nd.z));
    vec2 crd = vec2(0);
    float dist = 0.0;
    int offset = -1;
    // Низ
    if (lv.z < -limit) {
        crd = lv.xy * vec2(-0.5, -0.5) + 0.5;
        dist = -direction.z;
        offset = 5;
    }
    // Верх
    else if (lv.z > limit) {
        crd = lv.xy * vec2(0.5, -0.5) + 0.5;
        dist = direction.z;
        offset = 4;
    }
    // Зад
    else if (lv.y < -limit) {
        crd = lv.xz * vec2(0.5, -0.5) + 0.5;
        dist = -direction.y;
        offset = 3;
    }
    // Перед
    else if (lv.y > limit) {
        crd = lv.xz * 0.5 + 0.5;
        dist = direction.y;
        offset = 2;
    }
    // Лево
    else if (lv.x < -limit) {
        crd = lv.zy * vec2(0.5, -0.5) + 0.5;
        dist = -direction.x;
        offset = 1;
    }
    // Право
    else if (lv.x > limit) {
        crd = lv.zy * vec2(-0.5, -0.5) + 0.5;
        dist = direction.x;
        offset = 0;
    }
    
    if (offset >= 0) {
        float light_range = z_far - z_near;
        float cos_a = dot(fragment_normal, direction/dist);
        // Борьба с z-борьбой
        // Если свет падает под прямым углом, то z-fight почти не проявляется.
        // Чем больше вектор луча света приближается к касательной поверхности,
        // тем больше проявляется z-fight. Надо как-то считать смещение (bias),
        // которое бы уменьшило этот эффект.
        // В идеале хотелось бы брать тангенс от угла между лучом света и нормалью поверхности.
        // Однако, прежде, чем взять тангенс придётся ещё взять арккосинус, а это будет прожорливо.
        // И чёрт его знает, что там происходит у драйвера под капотом.
        //float bias = tan(acos(cos_a)) / light_range;
        // Можно заменить арккосинус с тангенсом на эту формулу.
        // Её график заметно отличается от acos(tg(a)), но обладает главным необходимым свойством:
        // при cos_a имеет небольшое значение и, когда cos_a прилижается к 0, значение bias
        // устремляется в бесконечность.
        float bias = abs(1.0/(cos_a-0.075)-1.175) * (light_range+5.0) * 0.00035 * (dist+1.0);
        float sampl = texelFetch(tex, ivec3(crd*ts, layer*6 + offset), 0).r;
        sampl = linearize_depth(sampl, z_near, z_far);
        return float(dist-bias-bias*length(dir_offset) < sampl);
    } else {
        return 1.0;
    }
}

#endif