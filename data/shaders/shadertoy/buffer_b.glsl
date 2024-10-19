void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    vec3 big_kernel   = blur(iChannel0, ivec2(fragCoord), BLUR_RADIUS);
#ifdef SHARPENING
    vec3 small_kernel = blur(iChannel0, ivec2(fragCoord), BLUR_RADIUS-1);
    fragColor.rgb = small_kernel + (small_kernel - big_kernel) * SHARPENING;
#else
    fragColor.rgb = big_kernel;
#endif
}