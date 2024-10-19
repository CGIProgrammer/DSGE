float hash1(inout float seed) {
    return fract(sin(seed += 0.1)*43758.5453123);
}

vec2 hash2(inout float seed) {
    return fract(sin(vec2(seed+=0.1,seed+=0.1))*vec2(43758.5453123,22578.1459123));
}

vec3 hash3(inout float seed) {
    return fract(sin(vec3(seed+=0.1,seed+=0.1,seed+=0.1))*vec3(43758.5453123,22578.1459123,19642.3490423));
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    float seed = fract(1.12345314312*iTime);
    vec2 disp = hash2(seed);
    float noise = hash1(seed);
    noise = texelFetch(iChannel1, ivec2(mod(fragCoord + disp*500.0, vec2(textureSize(iChannel1, 0).xy))), 0).r;
    // Coordinates for channel, in texel space.
    vec2 uv = vec2(fragCoord / vec2(iResolution.xy));
    vec3 col = texture(iChannel0, uv).rgb;
    //col *= mix(noise*2.0, 1.0, 0.0);
    col += (noise-0.5) * NOISE_MULTIPLIER;
    /*col.x = col.x > 0.5 ? 1.0 : 0.0;
    col.y = col.y > 0.5 ? 1.0 : 0.0;
    col.z = col.z > 0.5 ? 1.0 : 0.0;*/
    // Output to screen
    fragColor = vec4(clamp(col, -0.0, 1.0),1.0);
}