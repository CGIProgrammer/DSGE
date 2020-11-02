#include "data/shaders/functions/extensions.glsl"

uniform sampler2D channel;
uniform float width, height;
uniform int iFrame;
input vec2 tex_map;

vec2 hash21(float p)
{
	vec3 p3 = fract(vec3(p) * vec3(.1031, .1030, .0973));
	p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.xx+p3.yz)*p3.zy);

}

float hash13(vec3 p3, int component)
{
	p3  = fract(p3 * .1031 * float(component+1));
    p3 += dot(p3, p3.yzx + 19.19 - float(component));
    return fract((p3.x + p3.y) * p3.z);
}

#define R2 19

#define SIGMA 1.414
#define M_PI 3.14159265359

float gaussian (float x, float sigma) {
    float h0 = x / sigma;
    float h = h0 * h0 * -0.5;
    float a = 1.0 / (sigma * sqrt(2.0 * M_PI));
    return a * exp(h);
}

float distf(float v, float x) {
    return 1.0 - x;
}

vec2 quantify_error (sampler2D channel, int component, ivec2 p, ivec2 sz, float val0, float val1) {
    float Rf = float(R2) / 2.0;
    int R = int(Rf);
    float has0 = 0.0;
    float has1 = 0.0;
    float w = 0.0;
    
    for (int sy = -R; sy <= R; ++sy) {
        for (int sx = -R; sx <= R; ++sx) {
            float d = length(vec2(sx,sy));
            if ((d > Rf) || ((sx == 0) && (sy == 0)))
                continue;
            ivec2 t = (p + ivec2(sx,sy) + sz) % sz;
			float v = texelFetch(channel, t, 0)[component];

            float q = gaussian(d, SIGMA);
            has0 += (1.0-abs(v - val0)) * q;
            has1 += (1.0-abs(v - val1)) * q;
            w += q;
        }
    }
    vec2 result = vec2(has0 / w, has1 / w);
    return result;
}

void main()
{
    vec2 szf = vec2(width, height);
    ivec2 sz = ivec2(szf);
    ivec2 p0 = ivec2(tex_map*szf);
    vec2 maskf = hash21(float(iFrame));
    int M = 60 * 60;
    int F = (iFrame % M);
    float framef = float(F) / float(M);
    float chance_limit = 0.5;
    float force_limit = 1.0 - clamp(framef * 8.0, 0.0, 1.0);
    vec3 vals = vec3(0.0);
    force_limit = force_limit * force_limit;
    force_limit = force_limit * force_limit;
    for (int cmp=0; cmp<3; cmp++) {
        if (F == 0) {
            int c = (p0.x * 61 + p0.y) % 256;
            vals[cmp] = float(c) / 255.0;
        } else {
            ivec2 mask = ivec2(maskf * vec2(sz) + maskf * vec2(sz) * framef);
            ivec2 p1 = (p0 ^ mask) % sz;       
            ivec2 pp0 = (p1 ^ mask) % sz;  

            float chance0 = hash13(vec3(p0, float(iFrame)), cmp);
            float chance1 = hash13(vec3(p1, float(iFrame)), cmp);
            float chance = max(chance0, chance1);
            
            float v0 = texelFetch(channel, p0, 0)[cmp];
            float v1 = texelFetch(channel, p1, 0)[cmp];
            
            vec2 s0_x0 = quantify_error(channel, cmp, p0, sz, v0, v1);
            vec2 s1_x1 = quantify_error(channel, cmp, p1, sz, v1, v0);
            
            float err_s = s0_x0.x + s1_x1.x;
            float err_x = s0_x0.y + s1_x1.y;
            
            vals[cmp] = v0;
            if (pp0 == p0) {
                if ((chance < force_limit) || ((chance < chance_limit) && (err_x < err_s))) {
                    vals[cmp] = v1;
                }
            }
        }
    }
    fragColor = vec4(vals, 1.0);
}
