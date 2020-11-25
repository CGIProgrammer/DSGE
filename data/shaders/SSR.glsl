#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gAlbedo;
uniform sampler2D gSpace;
uniform sampler2D gMasks;
uniform sampler2D gAmbient;
uniform sampler2D gOutput;
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

#include "data/shaders/functions/random.glsl"

vec2 SSR_BS(vec3 rayHit, vec3 dir, int steps)
{
  vec4 projectedPosition;
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

vec4 SSR(vec3 rayHit, vec3 reflection, vec3 environment, float steps, float distance)
{
    float de = gRenderDepth(gSpace, tex_map);
    float max_dist = max(distance, de*5.0);
    float min_dist = max(max_dist * 0.002, 0.01);
    
    float dd = max_dist/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    mat4 camt = vCameraTransformInv*vCameraProjection;
    float delta,pd=0.0;
    float intensity = 1.0;
    vec3 currentHit = vec3(0.0);
    float look_norm = clamp(dot(-V, mNormal)*0.5+0.5, 0.0, 1.0);
    
    for (float d=min_dist;d<max_dist;d*=dd2)
    {
        delta = d-pd;
        vec3 mlrefl = reflection*d;
        vec4 mlRayHit = vec4(rayHit + mlrefl, 1.0);
        
        projectedPosition = mlRayHit * camt;
        UV = projectedPosition.xy / projectedPosition.w;
        UV = UV*0.5+0.5;
        float rd = gRenderDepth(gSpace, UV);
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            return vec4(environment.rgb, 200.0);
        }
        else if (projectedPosition.w>rd)
        {
            vec3 nrm = gRenderNormal(gSpace, UV);
            float nrd = 4.0 / (max(dot(nrm, V)*2.0, 0.0) + 0.4);
			intensity = projectedPosition.w-rd;
            if (intensity < delta*nrd)
            {
				float att = 1.0;
                vec2 fading = vec2(10.0);
                fading.y = fading.x*gResolution.y/gResolution.x;

                UV = SSR_BS(mlRayHit.xyz,mlrefl,5);
                att *= clamp(UV.x * fading.x, 0.0, 1.0);
                att *= clamp(UV.y * fading.y, 0.0, 1.0);
                att *= clamp((1.0-UV.x) * fading.x, 0.0, 1.0);
                att *= clamp((1.0-UV.y) * fading.y, 0.0, 1.0);
                vec3 refl = texture(gOutput, UV).rgb*texture(gAlbedo, UV).rgb;
                //refl = vec3(max(dot(gRenderNormal(gSpace, UV), V), 0.0));
                return vec4(mix(environment, refl, clamp(att,0.0,1.0)), d);
            }
            else
            {
                //continue;
            }
        }
        pd = d;
    }
    return vec4(environment, 200.0);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void main()
{
    if (texture(gAlbedo, tex_map).a < 0.5)
    {
      fragColor = vec4(0.0, 0.0, 0.0, 1.0); //texture(gOutput, tex_map);
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
    F0 = mix(vec3(0.04), vec3(1.0), mMetallic);
    float HV = dot(V, mNormal);
    vec3 F = F0 + (1.0 - F0) * pow(1.0 - clamp(HV, 0.0, 1.0), 5.0);
    fragColor = texture(gOutput, tex_map);
    //if (mMetallic>0.0)
    {
        vec3 reflectedVector = reflect(-V, mNormal);
        vec3 cbm = vec3(1.0);
        vec4 ssr = vec4(0.0);
        float rc = clamp(mRoughness/0.75, 0.0, 1.0);
        int cnt = int(round(mix(1.0, 1.0, rc)));
        float weight = 0.0;
        for (int i=0; i<cnt; i++) {
            vec3 noise = blueRand3(tex_map, i + gDitherIteration*cnt);
            vec3 vector = reflect(-V, normalize(mNormal + noise*0.2*mRoughness));
            float dott = dot(vector, mNormal);
            vec4 smp;
            cbm = textureCubemap(cubemap, vector*vec3(1.0,1.0,1.0), clamp(rc*5.0, 0.0, 10.0)).rgb * 2.0;
            if (mRoughness>0.75 || dott<0.0 || HV<-0.1)
            {
                smp.rgb = cbm;
            }
            else
            {
                vector += reflectedVector * ((noise.x+1.0)*(mRoughness+1.0)*0.5 + 0.02);
                smp = max(SSR(worldPosition.xyz, vector, cbm, mix(50.0, 2.0, rc), mix(10.0, 0.5, rc)), 0.0);
            }
            ssr.rgb += smp.rgb;
            weight += 1.0;
        }
        ssr.rgb /= float(cnt);
        if (isnan(F0.r) || isnan(F0.g) || isnan(F0.b)) {
            fragColor.rgb = vec3(1.0, 0.0, 1.0);
        } else {
            //fragColor.rgb = mix(fragColor.rgb, ssr.rgb, mix(F, vec3(0.0), rc));
            fragColor.rgb += mix(ssr.rgb * (1.0-rc), ssr.rgb*mDiffuse, clamp(mMetallic, 0.0, 1.0))*F;
        }
    }
}
