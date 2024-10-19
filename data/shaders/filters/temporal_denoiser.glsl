#include "../depth_packing.h"


ivec2 offsets[9] = ivec2[](
    ivec2(-1,  1), ivec2(0,  1), ivec2(1,  1),
    ivec2(-1,  0), ivec2(0,  0), ivec2(1,  0),
    ivec2(-1, -1), ivec2(0, -1), ivec2(1, -1)
);

vec2 camera_zrange;

void current_coords(sampler2D depth_sampler, ivec2 coords, vec2 fragCoord, out vec3 position, out float ndepth) {
    float normalized_depth = ndepth = texelFetch(depth_sampler, coords, 0).r;
    float linear_depth = linearize_depth(normalized_depth, camera_zrange.x, camera_zrange.y);
    
    position = gPosition(linear_depth, fragCoord, camera.projection_inverted, camera.transform).xyz;
}

void prev_coords(sampler2D depth_sampler, vec2 coords, out vec3 position, out float ndepth) {
    float normalized_depth = ndepth = texture(depth_sampler, coords).a;
    float linear_depth = linearize_depth(normalized_depth, camera_zrange.x, camera_zrange.y);
    
    position = gPosition(linear_depth, coords, camera.projection_inverted, camera.transform_prev).xyz;
}

float checker_mask(ivec2 pc, uint frame) {
    ivec2 dither[4] = ivec2[](
        ivec2(0, 0),
        ivec2(0, 1),
        ivec2(1, 1),
        ivec2(1, 0)
    );
    ivec2 checker_offset = (pc & 0x1) - dither[frame & 0x3];
    return float(checker_offset.x==0 && checker_offset.y==0);
}

void main() {
    camera_zrange = depth_range(camera.projection);
    vec2 half_res = textureSize(gDepth, 0);

    ivec2 pc = ivec2(pixelCoord.xy);

    ivec2 lowres_pc = pc / 2;
    noised_stack = texelFetch(noised, lowres_pc, 0);
    vectors_stack.xyz = texelFetch(gVectors, lowres_pc, 0).xyz;
    vectors_stack.w = texelFetch(gDepth, lowres_pc, 0).r;

    denoised_out = vec4(0, 0, 0, 1);
    
    float checker_mask0 = checker_mask(pc, timer.frame);
    float checker_mask1 = checker_mask(pc, timer.frame - 1);
    float checker_mask2 = checker_mask(pc, timer.frame - 2);
    float checker_mask3 = checker_mask(pc, timer.frame - 3);
    
    vec3 coords  = vec3(fragCoord.xy, vectors_stack.w);
    vec3 current_vectors = texture(gVectors,  coords.xy ).xyz;
    vec3 coords0 = coords;
    vec3 coords1 = coords0 - current_vectors;
    vec3 coords2 = coords1 - texture(gVectors1, coords1.xy).xyz;
    vec3 coords3 = coords2 - texture(gVectors2, coords2.xy).xyz;
    
    vec3 sample0 = texelFetch(noised , lowres_pc, 0).rgb;
    vec3 sample1 = texture   (noised1, coords1.xy).rgb;
    vec3 sample2 = texture   (noised2, coords2.xy).rgb;
    vec3 sample3 = texture   (noised3, coords3.xy).rgb;

    float ghosting_mask = distance(texelFetch(gVectors1, pc, 0).xyz, current_vectors) * 0.0;
    ghosting_mask += distance(texelFetch(gVectors2, pc, 0).xyz, current_vectors) * 0.0;
    ghosting_mask += distance(texelFetch(gVectors3, pc, 0).xyz, current_vectors) * 0.0;
    ghosting_mask = 1.0 / (1.0 + ghosting_mask);
    //ghosting_mask *= ghosting_mask*ghosting_mask;
    
    checker_mask0 = mix(1.0, checker_mask0, ghosting_mask);
    checker_mask1 = mix(0.0, checker_mask1, ghosting_mask);
    checker_mask2 = mix(0.0, checker_mask2, ghosting_mask);
    checker_mask3 = mix(0.0, checker_mask3, ghosting_mask);

    denoised_out.rgb = sample0 * checker_mask0;
    denoised_out.rgb+= sample1 * checker_mask1;
    denoised_out.rgb+= sample2 * checker_mask2;
    denoised_out.rgb+= sample3 * checker_mask3;
    if (coords0.x > 0.0) {
        //denoised_out.rgb = ghosting_mask.rrr;
    }
    //denoised_out.rgb = ghosting_mask.rrr;
    /*float h = pixelCoord.y / half_res.y * 0.5;
    if (h >= 0.0 && h <= 0.25) {
        denoised_out = texelFetch(noised, pc, 0);
    }
    if (h > 0.25 && h <= 0.5) {
        denoised_out = texelFetch(noised1, pc, 0);
    }
    if (h > 0.5 && h <= 0.75) {
        denoised_out = texelFetch(noised2, pc, 0);
    }
    if (h > 0.75) {
        denoised_out = texelFetch(noised3, pc, 0);
    }*/
    //denoised_out.rgb = h.rrr*h;
}

/*void main() {
    camera_zrange = depth_range(camera.projection);
    vec2 full_res = textureSize(denoised_in, 0);
    vec2 half_res = textureSize(gDepth, 0);

    ivec2 pc = ivec2(pixelCoord.xy);
    ivec2 lowres_pc = pc / 2;
    vec2 vectors = texelFetch(gVectors, lowres_pc, 0).xy;
    
    vec2 fragCoord_prev = fragCoord.xy - vectors.xy;
        
    vec4 prev = texture(denoised_in, fragCoord_prev);
    vec4 curr = texelFetch(noised, lowres_pc, 0);

    vec2 cdither = vec2(dither[timer.frame & 0x3]) - 0.5;
    vec2 pdither = vec2(dither[(timer.frame-1) & 0x3]) - 0.5;

    vec3 position, position_prev;
    //current_coords(gDepth, lowres_pc, fragCoord, position, curr.a);
    //prev_coords(denoised_in, fragCoord_prev, position_prev, prev.a);
    curr.a = texelFetch(gDepth, lowres_pc, 0).r;
    
    float normalized_depth = curr.a;
    float normalized_depth_prev = prev.a;

    float linear_depth = linearize_depth(normalized_depth, camera_zrange.x, camera_zrange.y);
    position = gPosition(linear_depth, fragCoord, camera.projection_inverted, camera.transform).xyz;

    float linear_depth_prev = linearize_depth(normalized_depth_prev, camera_zrange.x, camera_zrange.y);
    position_prev = gPosition(linear_depth_prev, fragCoord_prev, camera.projection_inverted, camera.transform_prev).xyz;
    
    float temporal_mask = max(distance(position, position_prev), 0.0) * length(vectors) * 10.0;

    ivec2 checker_offset = (pc & 0x1) - dither[timer.frame & 0x3];
    float checker_mask = float(checker_offset.x==0 && checker_offset.y==0);
    //temporal_mask *= 0.5;
    if (fragCoord_prev.x < 0.0 || fragCoord_prev.x > 1.0) {
        checker_mask = temporal_mask = 1.0;
    }
    if (fragCoord_prev.y < 0.0 || fragCoord_prev.y > 1.0) {
        checker_mask = temporal_mask = 1.0;
    }
    temporal_mask = pow(clamp(temporal_mask, 0.0, 1.0), 1.0);
    checker_mask = mix(checker_mask, 1.0, clamp(temporal_mask, 0.0, 1.0));
    denoised_out = mix(prev, curr, clamp(temporal_mask, 0.1, 1.0) * checker_mask);
    //denoised_out.rgb = length(temporal_mask).rrr * 10.0;
    swapchain_out = denoised_out; //vec4(vec3(temporal_mask), 1.0);
    swapchain_out.a = 1.0;
}*/
