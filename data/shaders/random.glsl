#ifndef RANDOM_H
#define RANDOM_H

#define TILE_SIZE 64
#define NOISE_TEXTURE_SIZE 1024
#define TILE_DIM (NOISE_TEXTURE_SIZE/TILE_SIZE)
#define TILES_COUNT (TILE_DIM*TILE_DIM)

// Функция bluerand4 возвращает сэмл синего шума для текущих координат
// Работатет только с текстурой шума 1024x1024
vec4 bluerand4(sampler2D noise_texture, int seed)
{
    ivec2 crd = ivec2(mod(gl_FragCoord.xy, vec2(TILE_SIZE)));
    seed = min(seed, TILES_COUNT);
    int seed_x = int(mod(seed, TILE_DIM));
    int seed_y = seed/TILE_DIM;
    return texelFetch(noise_texture, crd + ivec2(seed_x, seed_y) * TILE_SIZE, 0);
}

float InterleavedGradientNoise(vec2 pixel) 
{
    pixel += (float(timer.frame) * 5.588238f);
    return fract(52.9829189f * fract(0.06711056f*pixel.x + 0.00583715f*pixel.y));  
}

float InterleavedGradientNoise(vec2 pixel, int frame) 
{
    pixel += (float(frame) * 5.588238f);
    return fract(52.9829189f * fract(0.06711056f*pixel.x + 0.00583715f*pixel.y));  
}

float IGN(vec2 p, int frame)
{
    p += (float(frame) * 5.588238f);
    vec4 magic = vec4(0.06711056, 0.00583715, 0.5, 52.9829189);
    return fract( magic.w * fract(dot(p,magic.xy)) );
}

float IGN(vec3 p, int frame)
{
    p += (float(frame) * 5.588238f);
    vec4 magic = vec4(0.06711056, 0.00583715, 0.5, 52.9829189);
    return fract( magic.w * fract(dot(p,magic.xyz)) );
}

#endif