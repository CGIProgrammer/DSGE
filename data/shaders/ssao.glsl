#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gAlbedo;
uniform sampler2D gSpace;
uniform sampler2D gMasks;
uniform sampler2D gAmbient;
uniform sampler2D gOutput;
uniform sampler2D gLDiffuse, gLSpecular;
uniform samplerCube cubemap;
uniform vec2 gResolution;
uniform mat4 vCameraTransform;
uniform mat4 vCameraTransformInv;
uniform mat4 vCameraProjection;
uniform mat4 vCameraProjectionInv;
uniform int gDitherIteration;
vec3 V, mNormal, F0;
vec3 mDiffuse;
vec3 mAmbient;
float mSpecular;
float mRoughness;
float mMetallic;
vec4 projectedPosition;

#include "data/shaders/functions/random.glsl"

vec2 SSR_BS(vec3 rayHit, vec3 dir, int steps)
{
  vec2 UV;
  for (int i=0;i<steps;i++)
  {
    projectedPosition = vec4(rayHit,1.0)*vCameraTransformInv*vCameraProjection;
    UV = projectedPosition.xy / projectedPosition.w;
    UV = UV*0.5+0.5;
    
    float dDepth = projectedPosition.w - gRenderDepth(gSpace, UV);
    
    dir *= 0.5;
    if (dDepth>0.0)
    {
      rayHit -= dir;
    }
    else
    {
      rayHit += dir;
    }
    
    projectedPosition = vec4(rayHit,1.0)*vCameraTransformInv*vCameraProjection;
    UV = projectedPosition.xy / projectedPosition.w;
    UV = UV*0.5+0.5;
  }
  return UV;
}

/*vec4 SSRT2(vec3 rayHit, vec3 reflection)
{
    float de = abs(gRenderDepth(gSpace, tex_map));
    float min_dist = 0.02;
    float max_dist = min_dist*100.0; //clamp(min_dist, 0.5, 0.75);
    float steps = 10.0;
    float dd = (max_dist-min_dist)/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    
    float delta,pd=0.0;
    float intensity = 1.0;
    for (float d=min_dist;d<max_dist;d*=dd2)
    {
        delta = d-pd;
        vec3 mlrefl = reflection*d;
        vec4 mlRayHit = vec4(rayHit + mlrefl, 1.0);
        
        
        projectedPosition = mlRayHit*vCameraTransformInv*vCameraProjection;
        UV = projectedPosition.xy / projectedPosition.w;
        UV = UV*0.5+0.5;
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            continue;
        }
        else if (projectedPosition.w>gRenderDepth(gSpace, UV))
        {
            if (projectedPosition.w-gRenderDepth(gSpace, UV)< delta*2.7)
            {
                vec4 dynamicAO = vec4(texture(gOutput,SSR_BS(mlRayHit.xyz,mlrefl,6)).rgb, 1.0 / (1.0 + d*d));
                return dynamicAO;
            }
        }
        pd = d;
    }
    return vec4(0.0);
}*/

vec4 SSRT2(vec3 rayHit, vec3 reflection)
{
    float de = abs(gRenderDepth(gSpace, tex_map));
    float min_dist = 0.02;
    float max_dist = min_dist*100.0; //clamp(min_dist, 0.5, 0.75);
    float steps = 10.0;
    float dd = (max_dist-min_dist)/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    
    float delta,pd=0.0;
    float intensity = 1.0;
    for (float d=min_dist;d<max_dist;d*=dd2)
    {
        delta = d-pd;
        vec3 mlrefl = reflection*d;
        vec4 mlRayHit = vec4(rayHit + mlrefl, 1.0);
        
        projectedPosition = mlRayHit*vCameraTransformInv*vCameraProjection;
        UV = projectedPosition.xy / projectedPosition.w;
        UV = UV*0.5+0.5;
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            break;
        }
        else if (projectedPosition.w>gRenderDepth(gSpace, UV))
        {
            if (projectedPosition.w-gRenderDepth(gSpace, UV)< delta*2.7)
            {
                vec2 UV = SSR_BS(mlRayHit.xyz,mlrefl,3);
                vec3 diffuse = texture(gLDiffuse, UV).rgb;
                vec3 albedo = texture(gAlbedo, UV).rgb;
                vec4 dynamicAO = vec4(diffuse*albedo, 1.0 / (1.0 + d*d));
                return dynamicAO;
            }
        }
        pd = d;
    }
    return vec4(0.0);
}

vec2 kern[] = {
  vec2(0.70489159, 0.36926907),
  vec2(0.58508114, 0.29875646),
  vec2(0.04686315, 0.98178868),
  vec2(0.72980288, 0.30562532),
  vec2(0.95183668, 0.90224356),
  vec2(0.89178005, 0.10500403),
  vec2(0.80543446, 0.95344343),
  vec2(0.5717757 , 0.31319604),
  vec2(0.53027837, 0.04259763),
  vec2(0.8571465 , 0.05903962),
  vec2(0.6094955 , 0.74904762),
  vec2(0.40054468, 0.71567998),
  vec2(0.8205302 , 0.30937228),
  vec2(0.11145865, 0.53795805),
  vec2(0.09141175, 0.33078635),
  vec2(0.92441909, 0.96463933)
};

void main() {
    if (texture(gAlbedo, tex_map).a < 0.25)
    {
      /*fragColor = vec4(0.0, 0.0, 0.0, 1.0);
      return;*/
      discard;
    }
    vec4 worldPosition = gPosition(gSpace, tex_map, vCameraProjectionInv, vCameraTransform);
    vec4 masks = texture(gMasks, tex_map);
    mDiffuse = texture(gAlbedo, tex_map).rgb;
    mSpecular = masks.r;
    mRoughness = masks.g;
    mMetallic = masks.b;
    mAmbient = texture(gAmbient, tex_map).rgb;
    mNormal = gRenderNormal(gSpace, tex_map).rgb;
    V = normalize(transpose(vCameraTransform)[3].xyz - worldPosition.xyz);
    F0 = mix(vec3(0.04), mDiffuse.rgb, mMetallic);
    vec3 cbm = vec3(0.0);
    vec3 ao = vec3(0.0);
    int smpls = 1; //1+int(3 * mRoughness * (1.0-mMetallic));
    for (int i=0; i<smpls; i++) {
        //vec3 dir = normalize(blueRand3(tex_map+kern[gDitherIteration]/vec2(192.0, 108.0)*0.0, i+gDitherIteration*smpls));
        vec3 dir = normalize(normalize(blueRand3(tex_map+kern[gDitherIteration]/vec2(192.0, 108.0)*0.0, i+gDitherIteration)*0.5) + mNormal);
        //dir = mix(dir, mNormal, 0.5);
        float dn = dot(dir, mNormal);
        if (dn < 0.0)
        {
            dir *= -1.0;
            dn *= -1.0;
        }
        cbm = textureCubemap(cubemap, dir, 10.0).rgb;
        vec4 ssrt = max(SSRT2(worldPosition.rgb, dir), 0.0);
        ao += mix(cbm, ssrt.rgb, ssrt.a)*dn;
    }
    ao /= float(smpls);
    vec4 diff = texture(gLDiffuse, tex_map);
    float neg = 0.0;
    fragColor = vec4(diff.rgb+max(ao-neg, 0.0) / (1.0-neg), diff.a);
    fragColor.rgb *= 1.0-mMetallic;
    fragColor.a = 1.0;
}