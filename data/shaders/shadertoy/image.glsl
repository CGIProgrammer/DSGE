void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    ivec2 uv = ivec2(fragCoord*iChannelResolution[2].xy / vec2(iResolution.xy));
    if (fragCoord.y < iMouse.y) {
        if (fragCoord.x < iMouse.x) {
            fragColor.rgb = texelFetch(iChannel2, uv, 0).rgb;
        } else {
            vec3 big_kernel = blur(iChannel3, ivec2(fragCoord) , 5);
            vec3 small_kernel = blur(iChannel3, ivec2(fragCoord) , 4);
            fragColor.rgb = texelFetch(iChannel3, ivec2(fragCoord), 0).rgb;
            fragColor.rgb = fragColor.rgb + (small_kernel - big_kernel) * 0.0;
            fragColor.rgb = (fragColor.rgb - 0.5) * CONTRAST + 0.5;
        }
    } else {
        if (fragCoord.x > iMouse.x) {
            fragColor.rgb = texelFetch(iChannel0, ivec2(fragCoord), 0).rgb;
        } else {
            fragColor.rgb = texelFetch(iChannel1, ivec2(fragCoord), 0).rgb;
        }
    }
}