
vec3 texture_cube(samplerCube sampler,vec3 coords)
{
  return texture(sampler,vec3(coords.x, coords.z, -coords.y)).rgb;
}
vec3 texture_cube_lod(samplerCube sampler,vec3 coords, float lod)
{
  return textureLod(sampler,vec3(coords.x, coords.z, -coords.y), lod).rgb;
}

// float rand(vec2 co){
//     highp float a = 12.9898;
//     highp float b = 78.233;
//     highp float c = 43758.5453;
//     highp float dt= dot(co.xy ,vec2(a,b));
//     highp float sn= mod(dt,3.14);
//     return fract(sin(sn) * c);
// }
// 
// vec3 rand3(vec2 co)
// {
//   return vec3(rand(co),rand(co+10.0),rand(co+20.0));
// }
// 
// float blue_noise(vec2 coord)
// {
//     float pix = rand(coord);
//     return dFdx(pix);//sqrt(dFdx(pix)*dFdx(pix) + dFdy(pix)*dFdy(pix));
// }

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

vec4 SSR(vec3 rayHit, vec3 reflection, float steps)
{
    float de = abs(gRenderDepth(gNormals, tex_map));
    float max_dist = 10.0;//max(10.0, 10.0 * de);
    float min_dist = max_dist * 0.01;
    
    float dd = max_dist/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    //reflection *= dd;
    float delta,pd=0.0;
    float intensity = 1.0;
    vec3 cubemap_sample = texture_cube_lod(cubemap, reflection, 0.0).rgb;
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
            return vec4(cubemap_sample.rgb, 200.0);
        }
        else if (projectedPosition.w>gRenderDepth(gNormals, UV))
        {
            float rd = gRenderDepth(gNormals, UV);
            float nrd;
			intensity = projectedPosition.w-gRenderDepth(gNormals, UV);
            if (intensity < delta*1.7)
            {
				float attenuation = 1.0;
                vec2 fading = vec2(10.0);
                fading.y = fading.x*height/width;

                UV = SSR_BS(mlRayHit.xyz,mlrefl,8);
                attenuation *= clamp(UV.x * fading.x, 0.0, 1.0);
                attenuation *= clamp(UV.y * fading.y, 0.0, 1.0);
                attenuation *= clamp((1.0-UV.x) * fading.x, 0.0, 1.0);
                attenuation *= clamp((1.0-UV.y) * fading.y, 0.0, 1.0);
                //attenuation *= float(dot(reflection, gRenderNormal(gNormals, UV))<0.5);
                //nrd = gRenderDepth(gNormals, UV);
                //attenuation *= float(nrd > rd);
                
                //attenuation *= abs(nrd - rd);
                vec3 refl = texture(filtered, UV).rgb;
                return vec4(mix(cubemap_sample, refl, clamp(attenuation,0.0,1.0)), d);
            }
            else
            {
                //continue;
            }
        }
        pd = d;
    }
    //return vec4(0.25,0.25,0.25,1.0);
    return vec4(cubemap_sample, 200.0);
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
    float max_dist = min(5.0 * de,5.0);
    float min_dist = min(0.01 * de, 0.01);
    float steps = 10.0;
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
            //continue;
        }
        else if (projectedPosition.w>gRenderDepth(gNormals, UV))
        {
            if (projectedPosition.w-gRenderDepth(gNormals, UV)< delta*2.5)
            {
                vec4 dynamicAO = vec4(texture(original,SSR_BS(mlRayHit.xyz,mlrefl,5)).rgb,1.0 / (1.0 + d*0.2));
                return dynamicAO;
            }
        }
        pd = d;
    }
    return vec4(0.0);
}
/*
vec4 RT_VX(vec3 rayHit, vec3 reflection)
{
    float dist = 0.02;
    float max_dist = 1.0;
    float min_dist = dist;
    float steps = 5.0;
    float dd = (max_dist-min_dist)/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    //reflection *= dd;
    float delta,pd=0.0;
    float intensity = 1.0;
    for (dist=min_dist;dist<max_dist;dist*=dd2)
    {
        delta = dist-pd;
        vec3 mlrefl = reflection*dist;
        vec4 mlRayHit = vec4(rayHit + mlrefl, 1.0);
        
        
        projectedPosition = mlRayHit*camera_inverted*projection;
        UV = projectedPosition.xy / projectedPosition.w;
        UV = UV*0.5+0.5;
        if (UV.x>1.0 || UV.x<0.0 || UV.y>1.0 || UV.y<0.0)
        {
            break;
        }
        else if (projectedPosition.w>getRenderedDepth(UV))
        {
            if (projectedPosition.w-getRenderedDepth(UV)< delta*2.0)
            {
                return vec4(texture(original,SSR_BS(mlRayHit.xyz,mlrefl,5)).rgb, 1.0 / (1.0 + dist*dist));
            }
        }
        pd = dist;
    }
    
    dist = VOXEL_SIZE*4.0;
    max_dist = 10.0;
    min_dist = dist;
    steps = 35.0;
    dd2 = pow(max_dist/min_dist,1.0/steps);
    
    vec4 acc = vec4(0.0);
    for (;dist<max_dist;dist*=dd2)
    {
        float Lod = log2(1.0 + dist/VOXEL_SIZE*0.1);
        vec4 pos = vec4(rayHit + dist*reflection, 1.0);
        vec3 vs = voxelSpace(pos, VOXEL_MAP_SIZE);
        if (vs.x>1.0 || vs.x<0.0 || vs.y>1.0 || vs.y<0.0 || vs.z>1.0 || vs.z<0.0)
        {
            return vec4(texture_cube_lod(cubemap, reflection, 7.0),1.0);
        }
        vec4 voxelMapLayer = textureLod(gVoxelMap, vs, clamp(Lod,0.0, 3.0));
        if (voxelMapLayer.a>0)
        {
            vec4 val = voxelMapLayer/voxelMapLayer.a;
            val.a = 0.5;
            acc += vec4(val.xyz*val.a,val.a);
        }
        if (acc.a>=1.0)
        {
            acc.rgb /= acc.a;
            return vec4(acc.rgb, 1.0 / (1.0 + dist*0.2));
        }
    }
    //return vec4(texture_cube(cubemap, reflection),0.0);
    return vec4(texture_cube_lod(cubemap, reflection, 7.0),1.0);
}

vec4 RT_VX_only(vec3 rayHit, vec3 reflection)
{
    float max_dist = 20.0;
    float min_dist = 0.02;
    float steps = 20.0;
    float dd = (max_dist-min_dist)/steps;
    float dd2 = pow(max_dist/min_dist,1.0/steps);
    vec4 projectedPosition;
    vec2 UV;
    //reflection *= dd;
    float delta,pd=0.0;
    float intensity = 1.0;
    float dist;

    dist = VOXEL_SIZE*4.0;
    dd = (max_dist-dist)/steps;
    dd2 = pow(max_dist/dist,1.0/steps);
    vec4 acc = vec4(0.0);
    for (;dist<max_dist;dist*=dd2)
    {
        float Lod = log2(1.0 + dist/VOXEL_SIZE*0.1);
        vec4 pos = vec4(rayHit + dist*reflection, 1.0);
        vec3 vs = voxelSpace(pos, VOXEL_MAP_SIZE);
        if (vs.x>1.0 || vs.x<0.0 || vs.y>1.0 || vs.y<0.0 || vs.z>1.0 || vs.z<0.0)
        {
            return vec4(0.0);
        }
        vec4 voxelMapLayer = textureLod(gVoxelMap, vs, clamp(Lod,0.0, 3.0));
        if (voxelMapLayer.a>0)
        {
            vec4 val = voxelMapLayer/voxelMapLayer.a;
            val.a = 0.5;
            acc += vec4(val.xyz,val.a);
        }
        if (acc.a>=1.0)
        {
            acc.rgb /= acc.a;
            return vec4(acc.rgb, 1.0 / (1.0 + dist*0.2));
        }
    }
    return vec4(0.0);
    return vec4(texture_cube(cubemap, reflection),1.0);
}*/
