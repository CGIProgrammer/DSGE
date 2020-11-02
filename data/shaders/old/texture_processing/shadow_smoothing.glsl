#include "data/shaders/functions/extensions.glsl"

uniform sampler2D imageTexture;
input vec2 tex_map;
uniform float width, height;

vec2[] displacements = vec2[](
                                    vec2(-1.0, 3.0),vec2( 0.0, 3.0),vec2( 1.0, 3.0),
                    vec2(-2.0, 2.0),vec2(-1.0, 2.0),vec2( 0.0, 2.0),vec2( 1.0, 2.0),vec2( 2.0, 2.0),
    vec2(-3.0, 1.0),vec2(-2.0, 1.0),vec2(-1.0, 1.0),vec2( 0.0, 1.0),vec2( 1.0, 1.0),vec2( 2.0, 1.0),vec2( 3.0, 1.0),
    vec2(-3.0, 0.0),vec2(-2.0, 0.0),vec2(-1.0, 0.0),vec2( 0.0, 0.0),vec2( 1.0, 0.0),vec2( 2.0, 0.0),vec2( 3.0, 0.0),
    vec2(-3.0,-1.0),vec2(-2.0,-1.0),vec2(-1.0,-1.0),vec2( 0.0,-1.0),vec2( 1.0,-1.0),vec2( 2.0,-1.0),vec2( 3.0,-1.0),
                    vec2(-2.0,-2.0),vec2(-1.0,-2.0),vec2( 0.0,-2.0),vec2( 1.0,-2.0),vec2( 2.0,-2.0),
                                    vec2(-1.0,-3.0),vec2( 0.0,-3.0),vec2( 1.0,-3.0)
);
float[] weights = float[](
                   0.001, 0.001, 0.001,
            0.002, 0.012, 0.020, 0.012, 0.002,
    0.001,  0.002, 0.012, 0.020, 0.012, 0.002, 0.001,
    0.001,  0.020, 0.109, 0.172, 0.109, 0.020, 0.001,
    0.001,  0.002, 0.012, 0.020, 0.012, 0.002, 0.001,
            0.002, 0.012, 0.020, 0.012, 0.002,
                   0.001, 0.001, 0.001
);
void main() {
    fragColor = vec4(0.0);
    float totalWeight = 0.0;
    float min_z = 9999990.0;
    for (int i=0; i<37; i++)
    {
        vec2 disp  = displacements[i]/vec2(width, height);
        vec2 color = texture(imageTexture, tex_map + disp).rg;
        fragColor.rg += color.rg * weights[i];
        totalWeight += weights[i];
        min_z = min(color.r, min_z);
    }
    fragColor.rg /= totalWeight;
    fragColor.b = min_z;
    fragColor.a = 1.0;
};
