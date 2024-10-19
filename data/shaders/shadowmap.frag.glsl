#include "depth_packing.h"
void main()
{
    /*mat3 tbn = TBN;
    tbn[0] = normalize(tbn[0]);
    tbn[1] = normalize(tbn[1]);
    tbn[2] = normalize(tbn[2]);*/
    mNormal = vec3(0.0);
    mDiffuse.a = 1.0;
    principled();
    if (mDiffuse.a < 0.5)
    {
        mDiffuse.a = 1.0;
        discard;
    }
    //vec2 zrange = depth_range(camera.projection);
    //float dist = length(world_position);
    //vec2 d = (camera.projection * vec4(0.0, 0.0, -dist, 1.0)).zw;
    //gl_FragDepth = d.x / d.y;
}