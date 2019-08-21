
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
    float FOV,zFar,zNear;
};

uniform sSunLight lSun;
uniform sSpotLight lSpots[6];
uniform sPointLight lPoints[10];

uniform sampler2D lSpotShadowMaps[MAX_LIGHTS/2];
uniform sampler2D lSunShadowMap;

uniform int lSpotCount;
uniform int lPointCount;


const float shadow_res = 2048.0;
const float shadow_res_spot = 256.0;

//vec4 shadowCoords;

uniform sampler2D sdm[MAX_LIGHTS];

float getSpotShadowSample(int ind,vec2 coords)
{
    if (ind==0) return texture(lSpotShadowMaps[0], coords).r;
    if (ind==1) return texture(lSpotShadowMaps[1], coords).r;
    if (ind==2) return texture(lSpotShadowMaps[2], coords).r;
    if (ind==3) return texture(lSpotShadowMaps[3], coords).r;
    #if MAX_LIGHTS > 8
    if (ind==4) return texture(lSpotShadowMaps[4], coords).r;
    #if MAX_LIGHTS > 10
    if (ind==5) return texture(lSpotShadowMaps[5], coords).r;
    #if MAX_LIGHTS > 12
    if (ind==6) return texture(lSpotShadowMaps[6], coords).r;
    #if MAX_LIGHTS > 14
    if (ind==7) return texture(lSpotShadowMaps[7], coords).r;
    #endif
    #endif
    #endif
    #endif
    return 0.0;
}

float sunShadowSample(vec3 shadowCoords, vec2 displacement)
{
    float s_res = shadow_res;
    float zNear = lSun.zNear;
    float zFar = lSun.zFar;
    
    float z_b =  texture(lSunShadowMap, shadowCoords.xy+displacement).r;
    if (shadowCoords.x<=0.0 || shadowCoords.x>=1.0 ||
        shadowCoords.y<=0.0 || shadowCoords.y>=1.0) return 1.0;
    float sc = max(dot(normal, normalize(lSun.direction)), 0.1);
    return float(shadowCoords.z<z_b);
}

float spotShadowSample(int ind, vec3 shadowCoords, vec2 displacement)
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

float pixelSmoothShadowSun(vec3 shadowCoords){
   vec2 texSize = vec2(shadow_res);
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

    vec3 bi = vec3(0.0, 0.0, -0.002);

    float sample0 = sunShadowSample(shadowCoords + bi, delta.xz - shadowCoords.xy);
    float sample1 = sunShadowSample(shadowCoords + bi*0.5, delta.yz - shadowCoords.xy);
    float sample2 = sunShadowSample(shadowCoords + bi*0.5, delta.xw - shadowCoords.xy);
    float sample3 = sunShadowSample(shadowCoords + bi*0.1, delta.yw - shadowCoords.xy);

    float sx = s.x / (s.x + s.y);
    float sy = s.z / (s.z + s.w);

    return mix(
       mix(sample3, sample2, sx), mix(sample1, sample0, sx)
    , sy);
}

float pixelSmoothShadowSun2(vec3 shadowCoords)
{
    float s_res = shadow_res;
    float disp = 1.0/s_res;
    vec2 coords = (shadowCoords.xy)*s_res;
    vec2 remains = coords-vec2(ivec2(coords));
    float oo = sunShadowSample(shadowCoords, vec2(0.0));
    float oO = sunShadowSample(shadowCoords, vec2( 0.0, disp));
    float Oo = sunShadowSample(shadowCoords, vec2( disp,0.0));
    float OO = sunShadowSample(shadowCoords, vec2( disp));
    return mix(mix(oo,oO,remains.y), mix(Oo,OO,remains.y), remains.x);
}

float pixelSmoothShadowSpot(int ind, vec3 shadowCoords)
{
    float s_res = shadow_res_spot;
    float disp = 1.0/s_res;
    vec2 coords = (shadowCoords.xy)*s_res/shadowCoords.z;
    vec2 remains = (coords-vec2(ivec2(coords)));
    float oo = spotShadowSample(ind,shadowCoords,vec2( 0.0, 0.0));
    float oO = spotShadowSample(ind,shadowCoords,vec2( 0.0, disp));
    float Oo = spotShadowSample(ind,shadowCoords,vec2( disp,0.0));
    float OO = spotShadowSample(ind,shadowCoords,vec2( disp,disp));

    return mix(mix(oo,oO,remains.y), mix(Oo,OO,remains.y), remains.x);
}

#ifndef NO_LIGHTING
// Блики
vec3 glare(vec3 light_vector)
{
    vec3 e = normalize(world_position.xyz - vec3(camera_transform[0].w, camera_transform[1].w, camera_transform[2].w));
    vec3 r = normalize(-reflect(light_vector, normal));
    return max(vec3(1.0) * pow(max(dot(r, -e), 0.0), roughness),0.0) * specular_mask;
}

// Расчёт освещения для солнца
vec3 sunDiffuse()
{
    /*mat4 bbs = bias2;
    float sc = max(dot(normal, normalize(lSun.direction)), 0.0);
    bbs[2].w -= 0.002;*/
    mat4 lMVP = lSun.itransform*bias2;
    vec4 sh_coords = world_position*lMVP;
    float shadow_factor;
    
    shadow_factor = pixelSmoothShadowSun(sh_coords.xyz);
    //shadow_factor = pixelSmoothShadowSun2(sh_coords.xyz);
    
    float intensity;
    vec3 light;
    vec3 light_color;
    vec3 lightLookAt = normalize(lSun.direction);
    intensity = dot(normal,lightLookAt);
    light_color = pow(lSun.color.rgb,vec3(1.0/2.2));
    light = light_color * max(intensity,0.0)*lSun.color.w * shadow_factor;
    specular_factor += glare(lightLookAt) * light * 40.0;// * float(intensity!=0.0) * shadow_factor;
    return light;
}

// Расчёт освещения для точечных источников света
vec3 pointDiffuse()
{
    float intensity;
    vec3 result = vec3(0.0);
    vec3 light_color;
    vec3 light_posiiton;
    for (int ind=0;ind<lPointCount;ind++)
    {
        light_posiiton = lPoints[ind].position;
        float dist = length(lPoints[ind].position-world_position.xyz);
        intensity = dot(normal,normalize(lPoints[ind].position-world_position.xyz))/(1.0 + dist*dist); 
        light_color = pow(lPoints[ind].color.rgb,vec3(1.0/2.2))*lPoints[ind].color.w;
        result += light_color * max(intensity,0.0);
        specular_factor += glare(light_posiiton-world_position.xyz) * light_color;
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
        mat4 lMVP = lSpots[i].itransform*bias;//*perspective(1.0,1.0,lSpots[i].zNear,lSpots[i].zFar,lSpots[i].FOV)*bias;
        vec4 sh_coords = world_position*lMVP;
        
        float shadow_factor = pixelSmoothShadowSpot(i,sh_coords.xyw);
        
        light_posiiton = lSpots[i].position;
        light_distance = length(light_posiiton-world_position.xyz);
        light_vector = (light_posiiton-world_position.xyz)/light_distance;
        lightLookAt = normalize(lSpots[i].direction); 
        spotField = clamp(dot(lightLookAt,light_vector),0.0,1.0);
        if (spotField<lSpots[i].outer) continue;
        intensity = dot(normal,light_vector)/pow(1.0+light_distance*0.5,2.0);
        
        spotField = smoothstep(lSpots[i].outer,lSpots[i].inner,spotField);
        light_color = pow(lSpots[i].color.rgb,vec3(1.0/2.2))*lSpots[i].color.w;
        light = light_color * max(intensity,0.0) * spotField * shadow_factor;
        result += light;
        specular_factor += glare(light_posiiton-world_position.xyz)*intensity*lSpots[i].color.rgb*spotField * light_color * shadow_factor;
    }
    return result;
}
#endif
