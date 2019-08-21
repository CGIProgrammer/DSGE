
const float shadow_res = 2048.0;
const float shadow_res_spot = 256.0;

vec4 shadowCoords;

float getSpotShadowSample(int ind,vec2 coords)
{
    if (ind==0) return texture(lSpotShadowMaps[0], coords).r;
    if (ind==1) return texture(lSpotShadowMaps[1], coords).r;
    if (ind==2) return texture(lSpotShadowMaps[2], coords).r;
    if (ind==3) return texture(lSpotShadowMaps[3], coords).r;
    #if MAX_LIGHTS/2 >3
    if (ind==4) return texture(lSpotShadowMaps[4], coords).r;
    if (ind==5) return texture(lSpotShadowMaps[5], coords).r;
    if (ind==6) return texture(lSpotShadowMaps[6], coords).r;
    if (ind==7) return texture(lSpotShadowMaps[7], coords).r;
    #endif
    return 0.0;
}

float sunShadowSample(vec2 displacement)
{
    float s_res = shadow_res;
    float zNear = lSun.zNear;
    float zFar = lSun.zFar;
    
    float z_b =  texture(lSunShadowMap, shadowCoords.xy+displacement).r;
    if (shadowCoords.x<=0.0 || shadowCoords.x>=1.0 ||
        shadowCoords.y<=0.0 || shadowCoords.y>=1.0) return 1.0;
    return float(shadowCoords.z<z_b);
}

float spotShadowSample(int ind,vec2 displacement)
{
    float s_res = shadow_res_spot;
    float zNear = lSpots[ind].zNear;
    float zFar = lSpots[ind].zFar;
    float z_b =  getSpotShadowSample(ind,shadowCoords.xy/shadowCoords.z + displacement);
    float z_n = 2.0 * z_b - 1.0;
    float z_e = 2.0 * zNear * zFar / (zFar + zNear - z_n * (zFar - zNear));
    //return float(z_b*(zFar-zNear)+zNear > shadowCoords.z-0.3);
    return float(z_e > shadowCoords.z-0.1);
}

vec4 cubic(float v){
    vec4 n = vec4(1.0, 2.0, 3.0, 4.0) - v;
    vec4 s = n * n * n;
    float x = s.x;
    float y = s.y - 4.0 * s.x;
    float z = s.z - 4.0 * s.y + 6.0 * s.x;
    float w = 6.0 - x - y - z;
    return vec4(x, y, z, w) * (0.1666);
}

float pixelSmoothShadowSun(){

   vec2 texSize = textureSize(lSunShadowMap, 0);
   vec2 invTexSize = 1.0 / texSize;

   vec2 texCoords = shadowCoords.xy * texSize - 0.5;


    vec2 fxy = fract(texCoords);
    texCoords -= fxy;

    vec4 xcubic = cubic(fxy.x);
    vec4 ycubic = cubic(fxy.y);

    vec4 c = texCoords.xxyy + vec2 (-0.5, +1.5).xyxy;

    vec4 s = vec4(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
    vec4 delta = c + vec4 (xcubic.yw, ycubic.yw) / s;

    delta *= invTexSize.xxyy;

    float sample0 = sunShadowSample(delta.xz - shadowCoords.xy);
    float sample1 = sunShadowSample(delta.yz - shadowCoords.xy);
    float sample2 = sunShadowSample(delta.xw - shadowCoords.xy);
    float sample3 = sunShadowSample(delta.yw - shadowCoords.xy);

    float sx = s.x / (s.x + s.y);
    float sy = s.z / (s.z + s.w);

    return mix(
       mix(sample3, sample2, sx), mix(sample1, sample0, sx)
    , sy);
}

float pixelSmoothShadowSun2()
{
    float s_res = shadow_res;
    float disp = 1.0/s_res;
    vec2 coords = (shadowCoords.xy)*s_res;
    vec2 remains = coords-vec2(ivec2(coords));
    float oo = sunShadowSample(vec2(0.0));
    float oO = sunShadowSample(vec2( 0.0, disp));
    float Oo = sunShadowSample(vec2( disp,0.0));
    float OO = sunShadowSample(vec2( disp));

    return mix(mix(oo,oO,remains.y), mix(Oo,OO,remains.y), remains.x);
}

float pixelSmoothShadowSpot(int ind)
{
    float s_res = shadow_res_spot;
    float disp = 1.0/s_res;
    vec2 coords = (shadowCoords.xy)*s_res/shadowCoords.z;
    vec2 remains = (coords-vec2(ivec2(coords)));
    float oo = spotShadowSample(ind,vec2( 0.0, 0.0));
    float oO = spotShadowSample(ind,vec2( 0.0, disp));
    float Oo = spotShadowSample(ind,vec2( disp,0.0));
    float OO = spotShadowSample(ind,vec2( disp,disp));

    return mix(mix(oo,oO,remains.y), mix(Oo,OO,remains.y), remains.x);
}
