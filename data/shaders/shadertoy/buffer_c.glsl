void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    vec4 filter1 = vec4(0.0);
    vec4 filter2 = vec4(0.0);
    float s = 0.0;
    float props = 0.5;
#ifdef KUWAHARA
    kuwahara(filter1, fragCoord, iChannel0, iResolution);
#endif
#ifdef MEDIAN
    fastMedian(filter2, fragCoord, iChannel0, iResolution);
#endif
    fragColor.rgb = mix(filter1.rgb, filter2.rgb, props);
}