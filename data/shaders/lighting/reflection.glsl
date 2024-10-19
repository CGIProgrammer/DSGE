#include "../depth_packing.h"
#include "../constants.h"
#include "../random.glsl"
#include "pbr_structures.h"
precision highp float;

float gRenderDepth(sampler2D depthmap, vec2 crd, float z_near, float z_far) {
    return linearize_depth(texture(depthmap, crd).r, z_near, z_far);
}

float gRenderDepth(sampler2D depthmap, ivec2 crd, float z_near, float z_far) {
    return linearize_depth(texelFetch(depthmap, crd, 0).r, z_near, z_far);
}

vec2 SSR_BS(inout vec3 rayHit, vec3 dir, int steps, float z_near, float z_far)
{
  vec2 UV;
  vec3 dir_proj = dir;
  for (int i=0;i<steps;i++)
  {
    UV = rayHit.xy / rayHit.z;
    UV = UV*0.5+0.5;
    
    float dDepth = rayHit.z - gRenderDepth(gDepth, UV, z_near, z_far);
    
    dir_proj *= 0.5;
    if (dDepth>0.0)
    {
      rayHit -= dir_proj;
    }
    else
    {
      rayHit += dir_proj;
    }
  }
  UV = (rayHit.xy / rayHit.z) * 0.5 + 0.5;
  return UV;
}

#define SSR_LOG

mat4 camt = camera.projection * camera.transform_inverted;

vec4 SSR2(
    in sampler2D scene,
    in vec3 V,
    in vec3 rayOrigin,
    in vec3 rayDirect,
    in vec3 environment,
    in int steps,
    in int steps_bs,
    in float max_dist,
    in float min_dist,
    in float z_near,
    in float z_far,
    in float constant_thickness,
    out vec2 UV)
{
    float step, depth;
    vec3 ray_crd, nrm;
    ivec2 screen_crd;
    vec3 ray_origin = (camt * vec4(rayOrigin, 1.0)).xyw;
    vec4 ray_end = camt * vec4(rayOrigin + rayDirect, 1.0);
    if (constant_thickness < 0.0) {
        constant_thickness = 0.1;
    }
    // if (abs(ray_end.x/ray_end.w) > 1.0 || abs(ray_end.y/ray_end.w) > 1.0 ) {
    //     return vec4(environment, max_dist);
    // }
    vec3 ray_direct = (ray_end).xyw - ray_origin;
    vec2 texture_size = vec2(textureSize(gNormals, 0).xy);
    #ifdef SSR_LOG
    step = pow(max_dist/min_dist, 1.0/float(steps));
    for (float d=min_dist, pd=0.0; d<max_dist; pd=d, d*=step)
    #else
    step = (max_dist - min_dist) / float(steps);
    for (float d=min_dist, pd=0.0; d<max_dist; pd=d, d+=step)
    #endif
    {
        float z = ray_crd.z;
        ray_crd = ray_origin + ray_direct * d;
        float dz = max(constant_thickness, (ray_crd.z - z) * 2.0);
        UV = (ray_crd.xy / ray_crd.z) * 0.5 + 0.5;
        screen_crd = ivec2(UV * texture_size);
        if (screen_crd.x>=texture_size.x || screen_crd.x<0 || screen_crd.y>=texture_size.y || screen_crd.y<0) {
            return vec4(environment, max_dist);
        }
        depth = gRenderDepth(gDepth, screen_crd, z_near, z_far);
        if (ray_crd.z >= depth && ray_crd.z <= depth + dz && depth < z_far) {
            UV = SSR_BS(ray_crd, ray_direct * d, steps_bs, z_near, z_far);
            screen_crd = ivec2(UV * texture_size);
            vec4 alb = texelFetch(gAlbedo, screen_crd, 0);
            vec4 refl;
            // nrm = (camt * vec4(texelFetch(gNormals, screen_crd, 0).xyz, 0.0)).xyz;
            // vec3 real_ray_vector = ray_crd - ray_origin;
            // bool dist_clamp = distance(ray_origin, ray_crd) >= max_dist;
            // bool normal_clamp = dot(real_ray_vector, nrm) > 0.0;
            // if (normal_clamp) {
            //     continue;
            // }
            if (alb.a == 1.0) { // && dot(real_ray_vector, nrm) <= 0.0) {  // && sign(depth - ray_origin.z) == sign(ray_crd.z - ray_origin.z) && ) {
                float att = 1.0;
                vec2 fading = vec2(30.0);
                fading.y = fading.x*resolution.dimensions.y/resolution.dimensions.x;
                att *= clamp(UV.x * fading.x, 0.0, 1.0);
                att *= clamp(UV.y * fading.y, 0.0, 1.0);
                att *= clamp((1.0-UV.x) * fading.x, 0.0, 1.0);
                att *= clamp((1.0-UV.y) * fading.y, 0.0, 1.0);
                // att *= 1.0 / (1.0 + d * 4.0);
                // refl = vec4(alb.rgb, 1.0);
                refl = vec4(texelFetch(scene, screen_crd, 0).rgb * alb.rgb, 1.0);
                vec3 result = mix(environment, refl.xyz, clamp(att,0.0,1.0));
                return vec4(result, d);
            }
        }
    }
    return vec4(environment, max_dist);
}

vec4 SSR(
    sampler2D scene,
    vec3 V,
    vec3 rayHit,
    vec3 reflection,
    vec3 environment,
    int steps,
    int steps_bs,
    float max_dist,
    float min_dist,
    float z_near,
    float z_far,
    float constant_thickness,
    out vec2 UV)
{   
    float dd = max_dist/float(steps);
    float dd2 = pow(max_dist/min_dist,1.0/float(steps));
    mat4 camt = camera.projection * camera.transform_inverted;
    float delta,pd=0.0;
    float intensity = 1.0;
    vec3 currentHit = vec3(0.0);
    vec3 rayOrigin = (camt * vec4(rayHit, 1.0)).xyw;
    vec3 rayDirect = (camt * vec4(rayHit + reflection, 1.0)).xyw - rayOrigin;
    
    for (float d=min_dist;d<max_dist;d*=dd2)
    {
        delta = d-pd;
        vec3 mlrefl = rayDirect*d;
        vec3 mlRayHit = rayOrigin + mlrefl;
        
        UV = mlRayHit.xy / mlRayHit.z;
        UV = UV*0.5+0.5;
        float rd = gRenderDepth(gDepth, UV, z_near, z_far);
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            return vec4(environment.rgb, max_dist);
        }
        else if (mlRayHit.z>rd)
        {
            vec3 nrm = texture(gNormals, UV).xyz;
            float nrd = constant_thickness < 0.0 ?
                4.0 / (max(dot(nrm, V)*2.0, 0.0) + 1.0) :
                constant_thickness;
            if (mlRayHit.z-rd < delta*nrd)
            {
				float att = 1.0;
                vec2 fading = vec2(30.0);
                fading.y = fading.x*resolution.dimensions.y/resolution.dimensions.x;

                UV = SSR_BS(mlRayHit, mlrefl * 0.5, steps_bs, z_near, z_far);
                nrm = texture(gNormals, UV).xyz;
                if (dot(nrm, reflection) > 0.0) {
                    return vec4(environment, max_dist);
                }

                d = length(mlRayHit - rayOrigin);
                att *= clamp(UV.x * fading.x, 0.0, 1.0);
                att *= clamp(UV.y * fading.y, 0.0, 1.0);
                att *= clamp((1.0-UV.x) * fading.x, 0.0, 1.0);
                att *= clamp((1.0-UV.y) * fading.y, 0.0, 1.0);
                vec4 alb = texture(gAlbedo, UV);
                if (alb.a==1.0) {
                    vec3 refl = texture(scene, UV).rgb*alb.rgb;
                    return vec4(mix(environment, refl, clamp(att,0.0,1.0)), d);
                }
                else
                {
                    return vec4(environment.rgb, max_dist);
                }
            }
            else
            {
                //continue;
            }
        }
        pd = d;
    }
    return vec4(environment, max_dist);
}

vec3 CalculateKs( PBRSurface surface, vec3 V) {
    vec3 F0 = mix(vec3(0.04), surface.albedo, surface.metalness);
    float NdotV = max(0.0, dot(surface.normal, V));
    vec3 F = F0 + (1.0 - F0) * pow(1.0 - NdotV, 5.0);
    return F * surface.specular;
}

void CalculatePBR( PBRSurface surface, vec3 V, vec3 L, out vec3 diffuse, out vec3 specular) {
    float NdotV = max(0.0, dot(surface.normal, V));
    vec3 kS = CalculateKs(surface, V);
    specular = kS * float(NdotV > 0.0);
    diffuse = (1.0 - kS) * (1.0 - surface.metalness); // * float(NdotV > 0.0);
}

vec3 microsurface_reflection(vec3 normal, vec3 V, float roughness, vec3 noise_sample) {
    vec3 vector;
    if (dot(noise_sample, normal) < 0.0) {
        noise_sample = -noise_sample;
    }
    noise_sample = normalize(mix(normal, noise_sample, 0.99));
    vector = reflect(V, normal);
    vector = mix(vector, noise_sample, roughness);
    vector = normalize(vector);
    return vector;
}

float max(vec3 vec) {
    return max(max(vec.x, vec.y), vec.z);
}

void main()
{
    vec2 camera_zrange = depth_range(camera.projection);
    vec2 tex_map = pixelCoord / resolution.dimensions;
    ivec2 pixelCoord = ivec2(pixelCoord);
    
    float linear_depth = linearize_depth(texelFetch(gDepth, pixelCoord, 0).r, camera_zrange.x, camera_zrange.y);
    if (texelFetch(gDepth, pixelCoord, 0).r==1.0) {
        linear_depth = 1e14;
        specular_out = vec4(1.0);
        diffuse_out = vec4(1.0);
        return;
    }

    vec3 albedo = texelFetch(gAlbedo, pixelCoord, 0).rgb;
    vec3 normals = normalize(texelFetch(gNormals, pixelCoord, 0).rgb);
    vec3 position = gPosition(linear_depth, fragCoord, camera.projection_inverted, camera.transform).xyz;
    vec3 nV = normalize(camera.transform[3].xyz - position);
    vec4 masks = texelFetch(gMasks, pixelCoord, 0);

    PBRSurface surface;
    surface.position = position;
    surface.albedo = albedo;
    surface.normal = normals;
    surface.specular = masks.r;
    surface.roughness = clamp(masks.g, 0.0, 1.0);
    surface.metalness = masks.b;

    vec3 kS = CalculateKs(surface, nV) * surface.specular;
    vec3 F0 = mix(vec3(0.04), surface.albedo, surface.metalness);
    vec3 vector;
    diffuse_out = texelFetch(diffuse_in, pixelCoord, 0);
    specular_out = texelFetch(specular_in, pixelCoord, 0);
    vec3 ambient = vec3(0.01);
    float depth = gRenderDepth(gDepth, tex_map, camera_zrange.x, camera_zrange.y);

    float NdotV = max(dot(surface.normal, nV), 0.0);
    int samples = 8; //int(round(surface.roughness * 1.0 + 1.0));
    //surface.roughness = pow(surface.roughness, 0.1);

    bool diffuse_flag;
    float diffuse_flag_fl;

    int steps, steps_bs = 8;
    float max_dist, min_dist;
    float thickness;
    vec3 specular, diffuse, ssr_ambient;

    float kkS = 0.0;
    float kkD = 0.0;
    vec3 rtS = vec3(0.0);
    vec3 rtD = vec3(0.0);
    for (int i=0; i<samples; i++) {
        vec4 noise = bluerand4(blue_noise, int(mod((i + timer.frame*samples)/2, 64)));
        noise.xyz = normalize(noise.xyz * 2.0 - 1.0);
        diffuse_flag = bool(i & 1);
        //diffuse_flag = (surface.metalness > 0.5) ? false : (noise.a > kS.r);
        diffuse_flag_fl = float(diffuse_flag);
        
        if (diffuse_flag) {
            vector = microsurface_reflection(surface.normal, -nV, 1.0, noise.xyz);
        } else {
            vector = microsurface_reflection(surface.normal, -nV, surface.roughness, noise.xyz);
        }
        CalculatePBR(surface, nV, vector, diffuse, specular);
        bool skip_ray_march = (!diffuse_flag && max(kS) < 0.1) || (diffuse_flag && max(kS) > 0.9) || (dot(vector, nV) > 0.8);
        float ray_steps_coeff = abs(dot(-nV, vector));
        
        // Параметры SSRT для максимальной шероховатости (SSAO)
        float max_dist_diffuse = mix(0.5, 2.0, ray_steps_coeff);
        float min_dist_diffuse = max(max_dist_diffuse * 0.01, 0.01) + noise.a * 0.01;
        float thickness_diffuse = 0.5;
        float steps_bs_diffuse = mix(5.0, 10.0, ray_steps_coeff);
        float steps_diffuse = mix(5.0, 15.0, ray_steps_coeff); //8.0;
        
        // Параметры SSRT для минимальной шероховатости (SSR)
        float max_dist_specular = mix(3.0, camera_zrange.y*2.0, ray_steps_coeff);
        float min_dist_specular = max(max_dist_specular * 0.01, 0.01) + noise.a * 0.1;
        float thickness_specular = 1.0;
        float steps_bs_specular = mix(3.0, 8.0, ray_steps_coeff);
        float steps_specular = mix(8.0, 15.0, surface.roughness);
        
        vec2 end_ray_crd = tex_map;

        // Миксы параметров SSR
        max_dist = mix(max_dist_specular, max_dist_diffuse, diffuse_flag_fl);
        min_dist = mix(min_dist_specular, min_dist_diffuse, diffuse_flag_fl);
        thickness = mix(thickness_specular, thickness_diffuse, diffuse_flag_fl);
        steps_bs = int(round(mix(steps_bs_specular, steps_bs_diffuse, diffuse_flag_fl)));
        steps = int(round(mix(steps_specular, steps_diffuse, diffuse_flag_fl)));

        ssr_ambient = mix(ambient, ambient * surface.albedo, kS);
        vec4 ssr;
        if (skip_ray_march) {
            ssr = vec4(ssr_ambient, max_dist);
        }
        else {
            ssr = SSR2(diffuse_in, nV, position, vector, ssr_ambient, steps, steps_bs, max_dist, min_dist, camera_zrange.x, camera_zrange.y, thickness, end_ray_crd);
        }
        
        vec3 albedo_ray_hit = texture(gAlbedo, end_ray_crd).rgb;
        vec3 ssao_ambient = vec3(0.0);
        ssr_ambient = albedo_ray_hit * ssr_ambient * (1.0 - diffuse_flag_fl);
        ssr.rgb += (ssao_ambient + ssr_ambient) * float(ssr.a < max_dist);

        rtD += diffuse * ssr.rgb * (diffuse_flag_fl);
        rtS += specular * ssr.rgb * (1.0 - diffuse_flag_fl); //mix(ssr.rgb, ssr.rgb * surface.albedo, surface.metalness);
        kkD += diffuse_flag_fl;
        kkS += (1.0 - diffuse_flag_fl);
    }
    //float spec_mask = 1.0 - max(surface.specular, surface.metalness);
    if (kkD > 0.0) diffuse_out.rgb += rtD / kkD;
    if (kkS > 0.0) specular_out.rgb += rtS / kkS;
    //specular_out.rgb /= (max(surface.specular, surface.metalness) + 1.0/255.0);
}
