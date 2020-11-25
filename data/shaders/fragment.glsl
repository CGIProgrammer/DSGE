#include "data/shaders/functions/extensions.glsl"
#include "data/shaders/functions/projections.glsl"

input vec2 texture_uv;
input vec2 light_uv;
input mat3 TBN;
input vec3 view_vector;
input vec4 position;
input vec4 position_prev, position_stable;

uniform mat4 vCameraTransform;

vec4 worldPosition;
vec3 lightSample;
vec4 mDiffuse;
vec3 mNormal;
vec3 mAmbient;
vec3 F0;
float mSpecular;
float mRoughness;
float mMetallic;

#ifdef FORWARD
float width, height;
#include "data/shaders/functions/random.glsl"
#include "data/shaders/lighting.glsl"
#endif

//#define GAMMA

uniform sampler2D fDiffuseMap;
uniform vec4 fDiffuseValue;
uniform sampler2D fReliefMap;
uniform sampler2D fSpecularMap;
uniform sampler2D fLightMap;
uniform sampler2D fMetallicMap;
uniform sampler2D fRoughnessMap;

uniform float fSpecularValue;
uniform float fReliefValue;
uniform float fMetallicValue;
uniform float fRoughnessValue;
uniform float fFresnelValue;
uniform float fDistanceFormat;

#include "data/shaders/base_frag.glsl"

#if defined(SHADOW) && defined(FORWARD)
    #error SHADOW and FORWARD defined, which are alternatives
#endif

#if defined(SHADOW) && defined(DEFERRED)
    #error SHADOW and DEFERRED defined, which are alternatives
#endif

#if defined(FORWARD) && defined(DEFERRED)
    #error FORWARD and DEFERRED defined, which are alternatives
#endif

#if defined(FORWARD) && defined(DEFERRED) && defined(SHADOW)
    #error SHADOW, FORWARD and DEFERRED defined, which are alternatives
#endif

#if defined(Z_DISTANCE) && defined(VECTOR_DISTANCE)
    #error Z_DISTANCE and VECTOR_DISTANCE defined, which are alternatives
#endif

void main()
{
    worldPosition = position;
    mat3 tbn = transpose(TBN);
    tbn[0] = normalize(tbn[0]);
    tbn[1] = normalize(tbn[1]);
    tbn[2] = normalize(tbn[2]);
    mNormal = tbn[2];
    tbn = transpose(tbn);

    pbr(tbn);
    vec3 gamma = vec3(2.2);
    mDiffuse.rgb = pow(mDiffuse.rgb, gamma);
    if (mDiffuse.a < 0.1)
    {
        discard;
    }

#if defined(FORWARD)
    width = 1024.0;
    height = 600.0;
    mNormal = normalize(mNormal);
    F0 = vec3(0.04);
    #ifdef GAMMA
    mDiffuse.rgb = pow(mDiffuse.rgb, vec3(1.0/2.2));
    #endif
    vec3 lighting = lSunDiffuse() + mAmbient*mDiffuse.rgb;

	lighting += lSpotDiffuse();
	lighting += lPointDiffuse();

    fragColor.rgb = lighting;

    #ifdef GAMMA
    fragColor.rgb = pow(fragColor.rgb, vec3(2.2));
    fragColor.rgb = log(fragColor.rgb+1.0)*0.5;
    #endif
#else
    float dist = abs(mix(length(view_vector), abs(view_vector.z), fDistanceFormat));
#endif

#if defined(DEFERRED)
    fragData[0] = vec4(mDiffuse.rgb, 1.0);
    fragData[1] = vec4(mNormal*0.5+0.5, pack_depth(position.w));
    fragData[2] = vec4(mSpecular, mRoughness, mMetallic, fFresnelValue);
    fragData[3] = vec4(mAmbient, 1.0);
    fragData[4] = vec4((position_stable.xy/position_stable.w*0.5+0.5) - (position_prev.xy/position_prev.w*0.5+0.5), position_prev.z, pack_depth(position_prev.w));
#endif

#if defined(SHADOW)
    fragColor = vec4(dist, dist*dist, dist, 1.0);
#endif
}
