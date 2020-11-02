
vec3 voxelSpace(vec4 pos, float size)
{
    pos.z *= -1.0;
    vec3 coords = transpose(camera_transform)[3].xyz;
    coords = round(coords*32.0/size)/32.0*size;
    pos.xyz -= coords * vec3(1.0,1.0,-1.0);
    //pos *= inverse(vx_camera_transform);
    pos *= parallel(size,-size*0.5,size*0.5);
    return vec3(pos.xyz*0.5+0.5);
}

vec2 SSR_BS(vec3 rayHit, vec3 dir, int steps)
{
  vec4 projectedPosition;
  vec2 UV;
  for (int i=0;i<steps;i++)
  {
    projectedPosition = vec4(rayHit,1.0)*camera_inverted*projection;
    UV = projectedPosition.xy / projectedPosition.w;
    UV = UV*0.5+0.5;
    
    float dDepth = projectedPosition.w - gRenderDepth(gNormals, UV);
    
    dir *= 0.5;
    if (dDepth>0.0)
    {
      rayHit -= dir;
    }
    else
    {
      rayHit += dir;
    }
    
    projectedPosition = vec4(rayHit,1.0)*camera_inverted*projection;
    UV = projectedPosition.xy / projectedPosition.w;
    UV = UV*0.5+0.5;
  }
  return UV;
}

vec4 SSR(vec3 rayHit, vec3 reflection, float steps, float roughness)
{
    float de = abs(gRenderDepth(gNormals, tex_map));
    float max_dist = max(10.0, 10.0 * de*0.5);
    float min_dist = max_dist * 0.01;
    
    float dd = max_dist/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    //reflection *= dd;
    float delta,pd=0.0;
    float intensity = 1.0;
    vec3 spheremap_sample = textureCubemap(cubemap, reflection).rgb;
    vec3 currentHit = vec3(0.0);
    
    for (float d=min_dist;d<max_dist;d*=dd2)
    {
        delta = d-pd;
        vec3 mlrefl = reflection*d;
        vec4 mlRayHit = vec4(rayHit + mlrefl, 1.0);
        
        
        projectedPosition = mlRayHit*camera_inverted*projection;
        UV = projectedPosition.xy / projectedPosition.w;
        UV = UV*0.5+0.5;
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            return vec4(spheremap_sample.rgb, 200.0);
        }
        else if (projectedPosition.w>gRenderDepth(gNormals, UV))
        {
            float rd = gRenderDepth(gNormals, UV);
            float nrd;
			intensity = projectedPosition.w-gRenderDepth(gNormals, UV);
            if (intensity < pow(delta*1.7, 1.0))
            {
				float attenuation = 1.0;
                vec2 fading = vec2(10.0);
                fading.y = fading.x*height/width;

                UV = SSR_BS(mlRayHit.xyz,mlrefl,10);
                attenuation *= clamp(UV.x * fading.x, 0.0, 1.0);
                attenuation *= clamp(UV.y * fading.y, 0.0, 1.0);
                attenuation *= clamp((1.0-UV.x) * fading.x, 0.0, 1.0);
                attenuation *= clamp((1.0-UV.y) * fading.y, 0.0, 1.0);
                //attenuation *= clamp(dot(-reflection, gRenderNormal(gNormals, UV)),0,1);
                //nrd = gRenderDepth(gNormals, UV);
                //attenuation *= float(nrd > rd);
                
                //attenuation *= clamp(1.0 - roughness,0.0,1.0);
                vec3 refl = textureLod(filtered, UV, 0.0).rgb;
                return vec4(mix(spheremap_sample, refl, clamp(attenuation,0.0,1.0)), d);
            }
            else
            {
                //continue;
            }
        }
        pd = d;
    }
    //return vec4(0.25,0.25,0.25,1.0);
    return vec4(spheremap_sample, 200.0);
}

vec4 SSRT(vec3 rayHit, vec3 reflection)
{
  float de = abs(gRenderDepth(gNormals, tex_map));
  float max_dist = 0.1;
  float min_dist = 0.02 * de;
  float steps = 8.0 * de;
  float dd = (max_dist-min_dist)/steps;
  float dd2 = pow(max_dist/min_dist,1.0/steps);
  vec4 projectedPosition;
  vec2 UV;
  //reflection *= dd;
  float delta,pd=0.0;
  float intensity = 1.0;
  for (float d=min_dist;d<max_dist;d*=dd2)
  {
    delta = d-pd;
    vec3 mlrefl = reflection*d;
    vec4 mlRayHit = vec4(rayHit + mlrefl, 1.0);
    
    
    projectedPosition = mlRayHit*camera_inverted*projection;
    UV = projectedPosition.xy / projectedPosition.w;
    UV = UV*0.5+0.5;
    if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
    {
        continue;
    }
    else if (projectedPosition.w>gRenderDepth(gNormals, UV))
    {
      if (projectedPosition.w-gRenderDepth(gNormals, UV)< delta*1.5)
      {
        return vec4(texture(original,SSR_BS(mlRayHit.xyz,mlrefl,3)).rgb,1.0 / (1.0 + 0.2*d*d));
      }
    }
    pd = d;
  }
  return vec4(0.0);
}

vec4 SSRT2(vec3 rayHit, vec3 reflection)
{
    float de = abs(gRenderDepth(gNormals, tex_map));
    float min_dist = 0.01;
    float max_dist = min_dist*200; //clamp(min_dist, 0.5, 0.75);
    float steps = 5.0;
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
        
        
        projectedPosition = mlRayHit*camera_inverted*projection;
        UV = projectedPosition.xy / projectedPosition.w;
        UV = UV*0.5+0.5;
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            continue;
        }
        else if (projectedPosition.w>gRenderDepth(gNormals, UV))
        {
            if (projectedPosition.w-gRenderDepth(gNormals, UV)< delta*2.7)
            {
                vec4 dynamicAO = vec4(texture(original,SSR_BS(mlRayHit.xyz,mlrefl,3)).rgb, 1.0 / (1.0 + d*d*10.0));
                return dynamicAO;
            }
        }
        pd = d;
    }
    return vec4(0.0);
}