ivec2 detect_motion(sampler2D prev, ivec2 crd1, sampler2D curr, ivec2 crd2, int radius, out float diff)
{
    int stp = DETECT_MOTION_SNAKE_STEP;
    int n = 0;
    int square = radius*2+1;
    square *= square;
    int x=0,y=0;
    int dx=0,dy=-stp;
    ivec2 result = ivec2(0);
    diff = 65536.0;
    for (n = 0; n < square; n++) {
        //if ((-radius < x && x <= radius) && (-radius < y && y <= radius))
        {
            float d = compare_squares(prev, crd1 + ivec2(x, y), curr, crd2, SEARCH_KERNEL, diff);
            if (d < diff) {
                diff = d;
                result = ivec2(x, y);
            }
        }
        if (x == y || (x < 0 && x == -y) || (x > 0 && x == 1-y)) {
            int swap = dx;
            dx = -dy;
            dy =  swap;
            /*x += dx;
            y += dy;*/
        } else {
            x += dx;
            y += dy;
        }
    }
    return result;
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    
    float diff    = 0.0;
    ivec2 vector  = detect_motion(
        iChannel1, ivec2(fragCoord),
        iChannel0, ivec2(fragCoord/iResolution.xy*iChannelResolution[0].xy), SEARCH_RADIUS, diff);
    vec3 current  = texelFetch(iChannel0, ivec2(fragCoord), 0).rgb;
    
    vec3 big_kernel = blur(iChannel1, ivec2(fragCoord) + vector, 10);
    vec3 small_kernel = blur(iChannel1, ivec2(fragCoord) + vector, 1);
    float x = clamp(diff*SENSITIVITY, 0.0, 1.0);
    vec3 prev     =  texelFetch(iChannel1, ivec2(fragCoord) + vector, 0).rgb + (small_kernel - big_kernel) * (-0.8 + 0.1/(diff+0.1)); //
    fragColor.rgb = mix(prev, current, sqrt(x));
}