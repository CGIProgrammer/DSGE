#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gAlbedo;
uniform sampler2D gSpace;
uniform sampler2D gMasks;
uniform sampler2D gAmbient;
uniform samplerCube cubemap;
uniform float width, height;
uniform mat4 vCameraTransform;
uniform mat4 vCameraProjectionInv;

vec3 view_vector;
vec4 worldPosition;
vec3 lightSample;
vec4 mDiffuse;
vec3 mNormal;
vec3 mAmbient;
float mSpecular;
float mRoughness;
float mMetallic;
vec3 F0;
#include "data/shaders/functions/random.glsl"
#include "data/shaders/lighting.glsl"

void main()
{
    if (texture(gAlbedo, tex_map).a < 0.5) {
        fragData[0] = fragData[1] = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    vec3 gamma = vec3(1.0);
    vec4 masks = texture(gMasks, tex_map);
    mNormal = gRenderNormal(gSpace, tex_map).xyz;
    mAmbient = texture(gAmbient, tex_map).rgb;
    mDiffuse = texture(gAlbedo, tex_map);
    mSpecular = masks.r;
    mRoughness = masks.g;
    mMetallic = masks.b;
    F0 = mix(vec3(0.04), mDiffuse.rgb, mMetallic);

    worldPosition = gPosition(gSpace, tex_map, vCameraProjectionInv, vCameraTransform);
    view_vector = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
    //mDiffuse.rgb = pow(mDiffuse.rgb, gamma);
    fragData[0].rgb = fragData[1].rgb = vec3(0.0);
    lSunDiffuse(fragData[0].rgb, fragData[1].rgb);
    lSpotDiffuse(fragData[0].rgb, fragData[1].rgb);
    lPointDiffuse(fragData[0].rgb, fragData[1].rgb);
    fragData[0].a = fragData[1].a = 1.0;
}