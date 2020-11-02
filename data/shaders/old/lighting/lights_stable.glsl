
#define MAX_LIGHTS 16
//#define SUNLIGHT_SIZE 50.0
#define S_POINT 0
#define S_DIRECT 1
#define S_SUN 1
#define S_SPOT 2


struct sPointLight
{
    vec3 position;
    vec4 color;
};

struct sSpotLight
{
    vec3 position;
    vec4 color;
    vec3 direction;
    mat4 itransform;
    float inner;
    float outer;
    bool shadow;
    float FOV,zFar,zNear;
};

struct sSunLight
{
    vec3 direction;
    vec4 color;
    mat4 itransform;
    bool shadow;
    float FOV,zFar,zNear,size;
};

uniform sSunLight lSun;
uniform sSpotLight lSpots[MAX_LIGHTS];
uniform sPointLight lPoints[MAX_LIGHTS];

uniform sampler2D lSpotShadowMaps[MAX_LIGHTS];
uniform samplerCube lPointShadowMaps[MAX_LIGHTS];

uniform sampler2D lSunShadowMap, lSunShadowMapNear, lSunShadowMapNearDyn;

uniform int lSpotCount;
uniform int lPointCount;

const float shadow_res = 2048.0;
const float shadow_res_spot = 256.0;

uniform sampler2D sdm[MAX_LIGHTS];


vec4 fxaa(sampler2D shadow_map, vec2 fragCoord, vec2 resolution) {
    #ifndef FXAA_REDUCE_MIN
        #define FXAA_REDUCE_MIN   (1.0/ 128.0)
    #endif
    #ifndef FXAA_REDUCE_MUL
        #define FXAA_REDUCE_MUL   (1.0 / 8.0)
    #endif
    #ifndef FXAA_SPAN_MAX
        #define FXAA_SPAN_MAX     8.0
    #endif
    vec4 color;
    mediump vec2 inverseVP = vec2(1.0 / resolution.x, 1.0 / resolution.y);

    vec3 rgbNW;
    vec3 rgbNE;
    vec3 rgbSW;
    vec3 rgbSE;
    vec4 texColor;
    
    rgbNW = abs(texture(shadow_map, fragCoord + vec2(-1.0,  1.0)*inverseVP).rgb);
    rgbNE = abs(texture(shadow_map, fragCoord + vec2( 1.0,  1.0)*inverseVP).rgb);
    rgbSW = abs(texture(shadow_map, fragCoord + vec2(-1.0, -1.0)*inverseVP).rgb);
    rgbSE = abs(texture(shadow_map, fragCoord + vec2( 1.0, -1.0)*inverseVP).rgb);
    texColor = abs(texture(shadow_map, fragCoord));

    fragCoord.x*=resolution.x;
    fragCoord.y*=resolution.y;
    vec3 rgbM  = texColor.xyz;
    
    vec3 luma = vec3(0.299, 0.587, 0.114);
    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM,  luma);
    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    
    mediump vec2 dir;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));
    
    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) *
                          (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
              max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
              dir * rcpDirMin)) * inverseVP;
    
    vec3 rgbA;
    vec3 rgbB;

    rgbA = 0.5 * (
        abs(texture(shadow_map, fragCoord * inverseVP + dir * (1.0 / 3.0 - 0.5)).xyz) +
        abs(texture(shadow_map, fragCoord * inverseVP + dir * (2.0 / 3.0 - 0.5)).xyz));
    rgbB = rgbA * 0.5 + 0.25 * (
        abs(texture(shadow_map, fragCoord * inverseVP + dir * -0.5).xyz) +
        abs(texture(shadow_map, fragCoord * inverseVP + dir *  0.5).xyz));
    

    float lumaB = dot(rgbB, luma);
    if ((lumaB < lumaMin) || (lumaB > lumaMax))
        color = vec4(rgbA, texColor.a);
    else
        color = vec4(rgbB, texColor.a);
    return color;
}

vec4 getSpotShadowSample(int ind,vec2 coords)
{
    vec4 result = vec4(0.0);
    switch (ind) {
    case 0 : result = fxaa(lSpotShadowMaps[0], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    case 1 : result = fxaa(lSpotShadowMaps[1], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    case 2 : result = fxaa(lSpotShadowMaps[2], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    case 3 : result = fxaa(lSpotShadowMaps[3], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #if MAX_LIGHTS > 4
    case 4 : result = fxaa(lSpotShadowMaps[4], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 5
    case 5 : result = fxaa(lSpotShadowMaps[5], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 6
    case 6 : result = fxaa(lSpotShadowMaps[6], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 7
    case 7 : result = fxaa(lSpotShadowMaps[7], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 8
    case 8 : result = fxaa(lSpotShadowMaps[8], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 9
    case 9 : result = fxaa(lSpotShadowMaps[9], coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 10
    case 10: result = fxaa(lSpotShadowMaps[10],coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 11
    case 11: result = fxaa(lSpotShadowMaps[11],coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 12
    case 12: result = fxaa(lSpotShadowMaps[12],coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 13
    case 13: result = fxaa(lSpotShadowMaps[13],coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 14
    case 14: result = fxaa(lSpotShadowMaps[14],coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #elif MAX_LIGHTS > 15
    case 15: result = fxaa(lSpotShadowMaps[15],coords, vec2(shadow_res_spot, shadow_res_spot)); break;
    #endif
    }
    return abs(result);
}

vec4 getPointShadowSample(int ind,vec3 coords)
{
    switch (ind) {
    case 0 : return texture(lPointShadowMap[0], coords);
    case 1 : return texture(lPointShadowMap[1], coords);
    case 2 : return texture(lPointShadowMap[2], coords);
    case 3 : return texture(lPointShadowMap[3], coords);
    #if MAX_LIGHTS > 4
    case 4 : return texture(lPointShadowMap[4], coords);
    #elif MAX_LIGHTS > 5
    case 5 : return texture(lPointShadowMap[5], coords);
    #elif MAX_LIGHTS > 6
    case 6 : return texture(lPointShadowMap[6], coords);
    #elif MAX_LIGHTS > 7
    case 7 : return texture(lPointShadowMap[7], coords);
    #elif MAX_LIGHTS > 8
    case 8 : return texture(lPointShadowMap[8], coords);
    #elif MAX_LIGHTS > 9
    case 9 : return texture(lPointShadowMap[9], coords);
    #elif MAX_LIGHTS > 10
    case 10: return texture(lPointShadowMap[10],coords);
    #elif MAX_LIGHTS > 11
    case 11: return texture(lPointShadowMap[11],coords);
    #elif MAX_LIGHTS > 12
    case 12: return texture(lPointShadowMap[12],coords);
    #elif MAX_LIGHTS > 13
    case 13: return texture(lPointShadowMap[13],coords);
    #elif MAX_LIGHTS > 14
    case 14: return texture(lPointShadowMap[14],coords);
    #elif MAX_LIGHTS > 15
    case 15: return texture(lPointShadowMap[15],coords);
    #endif
    default : return vec4(0.0);
    }
}

float sunShadowSample(sampler2D shadowMap, vec3 shadowCoords,float variance_c)
{
    float f_b = abs(shadowCoords.z);
    
    //vec3 moments = abs(fxaa(shadowMap, shadowCoords.xy, vec2(2048.0, 2048.0)).rgb);
    vec3 moments = abs(texture(shadowMap, shadowCoords.xy).rgb);
    vec2 scxy = vec2(float(int(shadowCoords.x * 2048.0)), float(int(shadowCoords.y * 2048.0)));
    scxy = shadowCoords.xy*=2048.0;

    /*vec3 moments1 = abs(texture(shadowMap, (scxy.xy + vec2(-1.0, -1.0)*2.0) / 2048.0).rgb);
    vec3 moments2 = abs(texture(shadowMap, (scxy.xy + vec2( 1.0, -1.0)*2.0) / 2048.0).rgb);
    vec3 moments3 = abs(texture(shadowMap, (scxy.xy + vec2(-1.0,  1.0)*2.0) / 2048.0).rgb);
    vec3 moments4 = abs(texture(shadowMap, (scxy.xy + vec2( 1.0,  1.0)*2.0) / 2048.0).rgb);*/
    
    vec2 lit = vec2(0.0);
    
    float E_x2 = moments.y;
    float Ex_2 = moments.x * moments.x;
    float variance = (E_x2 - Ex_2);
    float mD = moments.x - f_b;
    float mD_2 = mD*mD;
    float p1 = variance / (variance * 0.025 + mD_2) * 0.025;
    float p2 = variance / (variance + mD_2);
    
    float bluredShadowSample1 = min(max(p1, float(f_b <= moments.x + 0.04)), 1.0);
    float bluredShadowSample2 = min(max(p2, float(f_b <= moments.x + 0.04)), 1.0);
    float regularShadowSample = float(f_b <= moments.z + 0.04);

    return min(bluredShadowSample2, regularShadowSample);
}

float spotShadowSample(int ind, vec4 shadowCoords, vec2 displacement)
{
    vec2 moments = abs(getSpotShadowSample(ind,shadowCoords.xy / shadowCoords.w + displacement).xy);
    float fz = abs(shadowCoords.z);
    vec2 lit = vec2(0.0);
    moments.x -= 0.01;
    float E_x2 = moments.y;
    float Ex_2 = moments.x * moments.x;
    float variance = E_x2 - Ex_2;
    float mD = moments.x - fz;
    float mD_2 = mD*mD;
    float p = variance / (variance + mD_2);
    
    return max(p, float(fz <= moments.x + 0.02));
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
/*
float pixelSmoothShadowSpot(int ind, vec4 shadowCoords)
{
    float s_res = shadow_res_spot;
    float disp = 1.0/s_res;
    vec2 coords = (shadowCoords.xy)*s_res/shadowCoords.w;
    vec2 remains = (coords-vec2(ivec2(coords)));
    float oo = 0.0;

    oo += spotShadowSample(ind,shadowCoords,vec2( -0.94201624, -0.39906216 ) / 256.0 * 0.7) * 0.25;
    oo += spotShadowSample(ind,shadowCoords,vec2( 0.94558609, -0.76890725 ) / 256.0 * 0.7) * 0.25;
    oo += spotShadowSample(ind,shadowCoords,vec2( -0.094184101, -0.92938870 ) / 256.0 * 0.7) * 0.25;
    oo += spotShadowSample(ind,shadowCoords,vec2( 0.34495938, 0.29387760 ) / 256.0 * 0.7) * 0.25;

    return oo;
}*/

#ifndef NO_LIGHTING
// Блики
vec3 glare(vec3 light_vector)
{
    float l = length(light_vector);
    vec3 e = normalize(world_position.xyz - vec3(camera_transform[0].w, camera_transform[1].w, camera_transform[2].w));
    vec3 r = -reflect(light_vector, normal)/l;
    return max(vec3(1.0) * pow(max(dot(r, -e), 0.0), 1.0/roughness),0.0) * specular_mask;
}

// Расчёт освещения для солнца
vec3 sunDiffuse()
{
    /*mat4 bbs = bias2;
    float sc = max(dot(normal, normalize(lSun.direction)), 0.0);
    bbs[2].w -= 0.002;*/
    //mat4 lMVP = lSun.itransform*bias;
    vec4 sh_coords = world_position * lSun.itransform;
    sh_coords.xy /= lSun.size*0.1;
    sh_coords.xy += 0.5;
    bool farShadow = sh_coords.x>1.0 || sh_coords.x<0.0 || sh_coords.y>1.0 || sh_coords.y<0.0;
    float far_shadow, near_shadow, shadow_factor;

    near_shadow = sunShadowSample(lSunShadowMapNearDyn, sh_coords.xyz, 1.0);
    near_shadow = min(near_shadow, sunShadowSample(lSunShadowMapNear, sh_coords.xyz, 1.0));

    sh_coords = world_position * lSun.itransform;
    sh_coords.xy /= lSun.size;
    sh_coords.xy += 0.5;
    
    far_shadow = sunShadowSample(lSunShadowMap, sh_coords.xyz, 1.0);
    
    shadow_factor = mix(near_shadow, far_shadow, float(farShadow));
    //shadow_factor = near_shadow;
    
    float intensity;
    vec3 light;
    vec3 light_color;
    vec3 lightLookAt = normalize(lSun.direction);
    intensity = dot(normal,lightLookAt);
    light_color = pow(lSun.color.rgb,vec3(1.0/2.2));
    light = light_color * max(intensity,0.0)*lSun.color.w * shadow_factor;
    specular_factor += glare(lightLookAt)*50.0 * light * float(intensity!=0.0) * shadow_factor;
    return light;
    //float z_b = -texture(lSunShadowMap, sh_coords.xy).r;
    //return vec3(float(z_b > f_b - 0.1));
    //return vec3(abs(z_b))*0.01;
    //return vec3(abs(f_b))*0.01;
}

// Расчёт освещения для точечных источников света
vec3 pointDiffuse()
{
    float intensity;
    vec3 result = vec3(0.0);
    vec3 light_color;
    vec3 light_posiiton;
    vec3 light_vector;
    for (int ind=0;ind<lPointCount;ind++)
    {
        light_posiiton = lPoints[ind].position;
        light_vector = light_posiiton-world_position.xyz;
        float dist = length(light_vector);
        intensity = max(dot(normal,normalize(light_vector))/(1.0 + dist*dist), 0.0); 
        light_color = pow(lPoints[ind].color.rgb,vec3(1.0/2.2))*lPoints[ind].color.w;
        light_vector = vec3(-light_vector.x, light_vector.z, -light_vector.y);
        
        vec3 moments = abs(getPointShadowSample(ind, light_vector/dist).rgb);
        vec2 lit = vec2(0.0);
        moments.x -= 0.01;
        float E_x2 = moments.y;
        float Ex_2 = moments.x * moments.x;
        float variance = (E_x2 - Ex_2) * 0.2;
        float mD = moments.x - dist;
        float mD_2 = mD*mD;
        float p = variance / (variance + mD_2);
        
        float shadow_factor = max(p, float(dist <= moments.x + 0.02));
        vec3 light_value = light_color * intensity*shadow_factor;
        result += light_value;
        specular_factor += glare(light_posiiton-world_position.xyz) * light_value;
    }
    return result;
}

// Расчёт освещения для прожекторов и фонариков
vec3 spotDiffuse()
{
    vec3 result = vec3(0.0);
    float spotField = 0.0;
    vec3 lightLookAt;
    vec3 light_posiiton;
    vec3 light_vector;
    vec3 light;
    vec3 light_color;
    float light_distance;
    float intensity;
    for (int i=0;i<lSpotCount;i++)
    {
        light_posiiton = lSpots[i].position;
        light_distance = length(light_posiiton-world_position.xyz);

        mat4 lMVP = lSpots[i].itransform*bias;
        vec4 sh_coords = world_position*lMVP;
        sh_coords.z = light_distance;
        
        float shadow_factor = spotShadowSample(i,sh_coords,vec2(0.0));
        float att = 1.0 / pow(1.0+light_distance*0.5,2.0);
        light_vector = (light_posiiton-world_position.xyz)/light_distance;
        lightLookAt = normalize(lSpots[i].direction); 
        spotField = clamp(dot(lightLookAt,light_vector),0.0,1.0);
        if (spotField<lSpots[i].outer) continue;
        intensity = dot(normal,light_vector) * att;
        
        spotField = smoothstep(lSpots[i].outer,lSpots[i].inner,spotField);
        light_color = pow(lSpots[i].color.rgb,vec3(1.0/2.2))*lSpots[i].color.w;
        light = light_color * max(intensity,0.0) * spotField * shadow_factor;
        result += light;
        specular_factor += glare(light_posiiton-world_position.xyz)*intensity*spotField * light_color * shadow_factor;
    }
    return result;
}
#endif
