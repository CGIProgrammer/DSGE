#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gAlbedo;
uniform sampler2D gSpace;
uniform sampler2D gMasks;
uniform sampler2D gAmbient;
uniform sampler2D gOutput;
uniform sampler2D gLDiffuse, gLSpecular;
uniform samplerCube cubemap;
uniform float width, height;
uniform mat4 vCameraTransform;
uniform mat4 vCameraTransformInv;
uniform mat4 vCameraProjection;
uniform mat4 vCameraProjectionInv;
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
vec4 SSRT2(vec3 rayHit, vec3 reflection)
{
    float de = abs(gRenderDepth(gSpace, tex_map));
    float min_dist = 0.025;
    float max_dist = min_dist*100.0;
    float steps = 7.0;
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
                vec2 UV = SSR_BS(mlRayHit.xyz,mlrefl,7);
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

void main() {
    if (texture(gAlbedo, tex_map).a < 0.5)
    {
      fragColor = vec4(0.0, 0.0, 0.0, 1.0);
      return;
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
    float HV = dot(V, mNormal);
    vec3 F = F0 + (1.0 - F0) * pow(1.0 - max(HV, 0.0), 5.0);
    vec3 reflectedVector = reflect(-V, mNormal);
    vec3 cbm = vec3(1.0);
    vec3 ao = vec3(0.0);
    int smpls = 4;
    for (int i=0; i<smpls; i++) {
        vec3 dir = normalize(normalize(blueRand3(tex_map, i))*0.9 + mNormal);
        float dn = dot(dir, mNormal);
        if (dn < 0.0)
        {
            dir *= -1.0;
        }
        cbm = textureCubemap(cubemap, dir, 10.0-float(i)*0.1).rgb * 2.0;
        vec4 ssrt = max(SSRT2(worldPosition.rgb, dir), 0.0);
        ao += mix(cbm, ssrt.rgb, ssrt.a)*dn;
    }
    ao /= float(smpls);
    vec4 diff = texture(gLDiffuse, tex_map);
    fragColor = vec4(diff.rgb+ao, diff.a);
    fragColor.rgb *= 1.0-mMetallic;
    fragColor.a = 1.0;
}