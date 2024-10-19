/*#ifndef CONTEXT

#endif*/
#include "constants.h"
#include "random.glsl"

void main()
{
    /*mat3 tbn = transpose(TBN);
    tbn[0] = normalize(tbn[0]);
    tbn[1] = normalize(tbn[1]);
    tbn[2] = normalize(tbn[2]);
    mNormal = vec3(0.0, 0.0, 1.0);
    tbn = transpose(tbn);*/
    mDiffuse.a = 1.0;
    mNormal = normalize(TBN[2].xyz);
    principled();
    mRoughness = max(mRoughness, 0.01);
    vec3 frag_coord = position.xyz/position.w*0.5+0.5;
    //if (mDiffuse.a < 0.99 && mDiffuse.a < IGN(vec3(frag_coord.xy*resolution.dimensions.xy, triangle_index * 0.01), int((timer.frame) & 0xFF)))
    if (mDiffuse.a < 0.5)
    {
        mDiffuse.a = 1.0;
        discard;
    }
    vec3 velocity_vector = vec3(frag_coord.xy, position.w) - vec3(position_prev.xy/position_prev.w*0.5+0.5, position_prev.w);
    gAlbedo.rgb = mDiffuse.rgb;
    gAlbedo.a = 1.0;
    gNormals = normalize(mNormal);
    gMasks = vec3(mSpecular, mRoughness, mMetallic);
    gVectors = vec4(velocity_vector, position_prev.w);
}